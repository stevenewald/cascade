use coordinate::{kafka_metadata_service_client::KafkaMetadataServiceClient, MetadataRequest};
use kafka_clone::proto_imports::coordinate;

mod publish {
    //this compiles the publish.proto file and generates a rust code for the gRPC services
    //we then import this rust code below
    tonic::include_proto!("publish");
}
use publish::publish_to_broker_client::PublishToBrokerClient;
use publish::my_api_service_server::{MyApiService, MyApiServiceServer};

use publish::PublishDataToBroker;
use publish::{ExpressDataToProducer, ProducerToExpressAck};

use tonic::{transport::Server, Request, Response, Status};

use prost_types::Timestamp;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

use std::sync::Arc;
use tokio::sync::Mutex;

use lazy_static::lazy_static;



struct CircularBuffer {
    buffer: Mutex<Vec<i32>>,
    write_ptr: Mutex<usize>,
    read_ptr: Mutex<usize>,
}

lazy_static! {
    static ref CIRCULAR_BUFFER: Arc<CircularBuffer> = Arc::new(CircularBuffer {
        buffer: Mutex::new(vec![0; 1000]),
        write_ptr: Mutex::new(0),
        read_ptr: Mutex::new(0),
    });
}

struct ProducerServer {
    // Get a ringbuffer
    producer_metadata: Arc<CircularBuffer>
}

impl ProducerServer {
    fn new(producer_metadata: Arc<CircularBuffer>) -> Self {
        

        ProducerServer {
            producer_metadata: producer_metadata
        }
    }
}

#[tonic::async_trait]
impl MyApiService for ProducerServer {
    async fn express_to_producer(
        &self,
        data_received: Request<ExpressDataToProducer>,
    ) -> Result<Response<ProducerToExpressAck>, Status> {
        
        // gRPC server setup and other initialization code
        //setup new grpc server (will not reuse a ton of code)
        //that receives ExpressDataToProducer and writes the string event
        // Receive request and data from gRPC server

        let mut buffer_lock = self.producer_metadata.buffer.lock().await;
        let mut write_ptr_lock = self.producer_metadata.write_ptr.lock().await;

        // Check if the next element in the buffer is 0
        if buffer_lock[*write_ptr_lock] == 0 {
            // Write the received data and advance write_ptr
            buffer_lock[*write_ptr_lock] = data_received.get_ref().data;
            *write_ptr_lock = (*write_ptr_lock + 1) % buffer_lock.len();
            Ok(tonic::Response::new(ProducerToExpressAck {
                response_to_express: 1,
            }))
        } else {
            // Error handling
            Ok(tonic::Response::new(ProducerToExpressAck { response_to_express: 0 }))
        }
            // The lock on the buffer will be automatically released when `buffer` goes out of scope
        }
    }
    


async fn sending(circular_buffer: Arc<CircularBuffer>) {
    //copy over a lot of code

    loop {
        let mut buffer_lock = circular_buffer.buffer.lock().await;
        let mut read_ptr_lock = circular_buffer.read_ptr.lock().await;

        // Check if the buffer length is greater than 0
        if buffer_lock.len() > 0 {
            // Pull element from the buffer at read_ptr and send it

            // Write 0 to the element at read_ptr
            buffer_lock[*read_ptr_lock] = 1;

            *read_ptr_lock = (*read_ptr_lock + 1) % buffer_lock.len();
        } else {
            // Error handling
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // the port 50001 result in connection error
    // creating a channel ie connection to server

    let mut backoff = 1;
    let coordinator_channel;
    loop {
        let result = tonic::transport::Channel::from_static("http://coordinator-service:50040")
            .connect()
            .await;

        match result {
            Ok(channel) => {
                println!("Connected successfully");
                coordinator_channel = Some(channel);
                break;  // or return, depending on your need
            },
            Err(e) => {
                println!("Failed to connect: {}. Retrying in {} seconds...", e, backoff);
                tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                if backoff < 60 {  // prevent the backoff time from getting too long
                    backoff *= 2;
                }
            }
        }
    }
    println!("Connected to coordinator");

    // creating gRPC client for KafkaMetadataService from channel
    let mut kafka_metadata_service_client = KafkaMetadataServiceClient::new(coordinator_channel.unwrap());

    // creating a new Request to send to KafkaMetadataService
    let metadata_request = tonic::Request::new(MetadataRequest {
        topic_name: "test".to_string(),
    });
    // sending metadata_request and waiting for response
    let metadata_response = kafka_metadata_service_client
        .get_metadata(metadata_request)
        .await?
        .into_inner();
    let num_partitions = metadata_response.brokers.len();
    println!(
        "Received metadata from coordinator with {} partitions.",
        num_partitions
    );

    //1. from received metadata, copy(clone()) the partitions
    // let partitions = metadata_response.brokers.clone(); //is there a function to access just the partition we want? or is cloning the whole response fine?

    //2. for each broker/partition (effectively same), establish a connection and put it in a vector
    let mut clients = Vec::new();
    for broker in &metadata_response.brokers {
        let usable_ip = format!("http://{}:{}", broker.ip, broker.port);
        println!("Connecting to {}", usable_ip);
        //let address = format!("{}:{}", broker.host, broker.port);
        let broker_channel = tonic::transport::Channel::from_shared(usable_ip)? //127.0.0.1:50050")? //steve todo:
            //should use the ip from the broker
            .connect()
            .await?;
        let client_connection_to_broker = PublishToBrokerClient::new(broker_channel);
        clients.push(client_connection_to_broker); //is client and broker the same in this case?
    }

    // tokio::spawn(async {
    //     express_to_producer(Arc::clone(&CIRCULAR_BUFFER)).await;
    // });

    tokio::spawn(async {
        sending(Arc::clone(&CIRCULAR_BUFFER)).await;
    });

    let events_to_send = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
    // let brokers = metadata_response.brokers;
    let mut partition_index = 0;

    while partition_index < events_to_send.len() {
        // //approach 1
        // let partition = partition_index % brokers.len();
        // let broker = brokers[partition];

        // //approach 2 that i think makes more sense?
        // get the client connection for the next partition using round-robin
        let mut client = clients[partition_index % clients.len()].clone(); //do i need to specify the broker still
                                                                           //or does client takes care of that when i'm doing client_connection_to_broker below?
                                                                           // get the partition metadata for the next partition
                                                                           // let partition_metadata = partitions[partition_index % partitions.len()].clone();

        // // converting Duration to Timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = Timestamp {
            seconds: now.as_secs() as i64,
            nanos: now.subsec_nanos() as i32,
        };

        // // creating a new Request to send to broker
        let mut rng = rand::thread_rng();
        let data_to_broker = tonic::Request::new(PublishDataToBroker {
            event_name: events_to_send[partition_index].to_string(),
            timestamp: Some(timestamp),
            number: rng.gen::<i32>(), // this is where the cpu usage (%) will go, make it a float though
        });
        // // sending data_to_broker and waiting for response
        let ack_from_broker = client.send(data_to_broker).await?.into_inner();
        println!("Broker ack {}", ack_from_broker.response_to_producer);

        partition_index += 1;
    }

    let addr = "0.0.0.0:50051".parse().unwrap();
    let service: ProducerServer = ProducerServer::new(Arc::clone(&CIRCULAR_BUFFER));
    Server::builder()
        .add_service(MyApiServiceServer::new(service))
        .serve(addr)
        .await?;

// server.await.unwrap();
    Ok(())
}

