use kafka_clone::proto_imports::coordinate as coordinate;
use coordinate::{kafka_metadata_service_client::KafkaMetadataServiceClient, MetadataRequest};

mod publish {
    //this compiles the publish.proto file and generates a rust code for the gRPC services
    //we then import this rust code below
    tonic::include_proto!("publish");
}
use publish::publish_to_broker_client::PublishToBrokerClient;
use publish::PublishDataToBroker;

use prost_types::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

use tokio::sync::Mutex;
use std::sync::Arc;


struct CircularBuffer {
    buffer: Vec<i32>,
    write_ptr: usize,
    read_ptr: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // the port 50001 result in connection error
    // creating a channel ie connection to server
    let coordinator_channel = tonic::transport::Channel::from_static("http://coordinator-service:50040")
        .connect()
        .await?;

    println!("Connected to coordinator");
    
    // creating gRPC client for KafkaMetadataService from channel
    let mut kafka_metadata_service_client = KafkaMetadataServiceClient::new(coordinator_channel);

    // creating a new Request to send to KafkaMetadataService
    let metadata_request = tonic::Request::new(MetadataRequest {
        topic_name: "test".to_string(),
    });
    // sending metadata_request and waiting for response
    let metadata_response = kafka_metadata_service_client.get_metadata(metadata_request).await?.into_inner();
    let num_partitions = metadata_response.brokers.len();
    println!("Received metadata from coordinator with {} partitions.", num_partitions);


    //1. from received metadata, copy(clone()) the partitions
    // let partitions = metadata_response.brokers.clone(); //is there a function to access just the partition we want? or is cloning the whole response fine?
    
    //2. for each broker/partition (effectively same), establish a connection and put it in a vector
    let mut clients = Vec::new();
    for broker in &metadata_response.brokers {
        let usable_ip = format!("http://{}:{}", broker.ip, broker.port);
        println!("Connecting to {}", usable_ip);
        //let address = format!("{}:{}", broker.host, broker.port);
        let broker_channel = tonic::transport::Channel::from_shared(usable_ip)?//127.0.0.1:50050")? //steve todo:
            //should use the ip from the broker
            .connect()
            .await?;
        let client_connection_to_broker = PublishToBrokerClient::new(broker_channel);
        clients.push(client_connection_to_broker); //is client and broker the same in this case? 
    }

    // events_to_send (made up) = [a, b, c, d]
    // brokers = a vector from metadata, ie.: [1, 2, 3, 4]
    // counter = 0
    // while counter < len(events_to_send):
        // brokers[counter % len(brokers)].send(events_to_send[counter])
        // counter+=1
    //3. use the above pseudocode to loop through as the pseudocode implies

    //rough outline of program
    //startup (already written)
    //define size of buffer arbitrarily (1000)
    //two async processes (always running)

    let circular_buffer = Arc::new(Mutex::new(CircularBuffer {
        buffer: vec![0; 1000],
        write_ptr: 0,
        read_ptr: 0,
    }));

    // tokio::spawn(async {
    //     receiving(circular_buffer.clone()).await;
    // });

    // tokio::spawn(async {
    //     sending(circular_buffer).await;
    // });

    // let buffer_size: usize = 1000;
    // let mut buffer: Vec<i32> = vec![0; buffer_size];
    // let mut write_ptr: usize = 0;

    //async process 1:
    //gRPC server that always listens for ExpressDataToProducer requests (similar to Broker)
    //when it receives a request, write_ptr checks if next element is 0
    //if it is, then write the received element and advance write_ptr
    //if not, error


    //async process 2:
    //constantly checks if the ring buffer length>0 (i.e., requests that havent been
    //sent to the brokers yet)
    //pull element from the ring buffer (at read_ptr) and send it
    //write 0 to the element at read_ptr
    //error it out if it's full for now
    





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
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
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

    // let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");

    // // converting Duration to Timestamp
    // let timestamp = Timestamp {
    //     seconds: now.as_secs() as i64,
    //     nanos: now.subsec_nanos() as i32,
    // };

    // // creating a new Request to send to broker
    // let mut rng = rand::thread_rng();
    // let data_to_broker = tonic::Request::new(PublishDataToBroker {
    //     event_name: String::from("default"),
    //     timestamp: Some(timestamp),
    //     number: rng.gen::<i32>(), // this is where the cpu usage (%) will go, make it a float though
    // });
    // // sending data_to_broker and waiting for response
    // let ack_from_broker = client_connection_to_broker.send(data_to_broker).await?.into_inner();
    // println!("Received acknowledgement from broker with message: {}", ack_from_broker.response_to_producer);

    // make a for loop that runs for 10k iterations (sleep .01 seconds after sending each event, so ~100hz)
    // send cpu usage (as a float assuming we can get good precision on cpu data) 100x per second
    // make it asynchronous (can remove the .await? to make it asynchronous)
    // look into whether there's a way (and if it's a good idea, maybe its not) to not have the broker send a response

    //this is what it would look like in python
    // for i in range(10000):
        // data_to_broker = ...
        // client_connection_to_broker.send(data_to_broker)
        // sleep(0.01)

    // if you remove the sleep functions, what happens (does it crash, if not, how many events/sec can it send)
    // also worth experimenting with using the same timestamp
    

    Ok(())
}

// async fn receiving(circular_buffer: Arc<Mutex<CircularBuffer>>) {
//     // gRPC server setup and other initialization code

//     loop {
//         // Receive request and data from gRPC server

//         let mut buffer = circular_buffer.lock().await;

//         // Check if the next element in the buffer is 0
//         if buffer.buffer[buffer.write_ptr]==0 {
//             // Write the received data and advance write_ptr
//             buffer.buffer[buffer.write_ptr] = 1;
//             buffer.write_ptr = (buffer.write_ptr + 1) % buffer.buffer.len();
//         } else {
//             // Error handling
//         }

//         // Release the lock on the buffer
//         drop(buffer);
//     }
// }

// async fn sending(circular_buffer: Arc<Mutex<CircularBuffer>>) {
//     loop {
//         let mut buffer = circular_buffer.lock().await;

//         // Check if the buffer length is greater than 0
//         if buffer.buffer.len() > 0 {
//             // Pull element from the buffer at read_ptr and send it

//             // Write 0 to the element at read_ptr
//             buffer.buffer[buffer.read_ptr] = 1;
//             buffer.read_ptr = (buffer.read_ptr + 1) % buffer.buffer.len();
//         } else {
//             // Error handling
//         }

//         // Release the lock on the buffer
//         drop(buffer);
//     }
// }