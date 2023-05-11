mod publish {
    //this compiles the publish.proto file and generates a rust code for the gRPC services
    //we then import this rust code below
    tonic::include_proto!("publish");
}

mod consume {
    tonic::include_proto!("consume");
}
use consume::consume_from_broker_server::{ConsumeFromBroker, ConsumeFromBrokerServer};
use consume::{BrokerToConsumerAck, ConsumeDataFromBroker, Event};
use publish::publish_to_broker_server::{PublishToBroker, PublishToBrokerServer};
use publish::{BrokerToPublisherAck, PublishDataToBroker};

use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;
use tonic::{transport::Server, Request, Response, Status};

use std::sync::Mutex;
use std::io::{Read, Write, Seek, SeekFrom};
use std::fs::OpenOptions;
use std::fs;

use dotenv;

//this function is just to print and parse the timestamp of the event we receive
fn timestamp_to_string(timestamp: Option<Timestamp>) -> String {
    if let Some(ts) = timestamp {
        if let Some(naive_datetime) = NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32)
        {
            let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
            datetime.to_rfc3339()
        } else {
            "Invalid timestamp".to_string()
        }
    } else {
        "No timestamp provided".to_string()
    }
}

// defining a struct for our publish to broker service (this side, the broker, is receiving and replying)
// #[derive(Default)]
struct BrokerServer {
    // eventually, will be multiple files, but just one for now
    // this file stores the actual events
    events_log: Mutex<std::fs::File>,
    index_table: Mutex<std::fs::File>,
}

impl BrokerServer {
    fn new() -> Self {
        let events_log = OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open("events.log")
            .expect("Unable to open events log");
        
        let index_table = OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open("index.table")
            .expect("Unable to open index table");
        println!("opened file!");

        BrokerServer {
            events_log: Mutex::new(events_log),
            index_table: Mutex::new(index_table),
        }
    }
}

// implementing rpc for publish to the service defined in publish.proto
#[tonic::async_trait]
impl PublishToBroker for BrokerServer {
    // our rpc impelemented as function
    async fn send(
        &self,
        data_received: Request<PublishDataToBroker>,
    ) -> Result<Response<BrokerToPublisherAck>, Status> {
        let mut index_table_file = self.index_table.lock().unwrap();
        let mut events_log_file = self.events_log.lock().unwrap(); //todo: could cause deadlocks?
        let event_to_string = &data_received.get_ref().event_name; //todo: make this more complex
        let start_index = events_log_file.seek(SeekFrom::Current(0))?;
        
        events_log_file.write(event_to_string.as_bytes()).expect("Failed to append event to events file\n");
        index_table_file.write(&(start_index.to_le_bytes())).expect("Failed to append index to index table file\n");
        println!("Created new file in log with index {}", start_index.to_string());
        println!("Received message from producer: {}", data_received.get_ref().event_name);
        Ok(Response::new(BrokerToPublisherAck {
            // reading data from request which is awrapper around our PublishDataToBroker message defined in .proto
            response_to_producer: format!(
                "successfully rx event with name {} and timestamp {} and number {}",
                data_received.get_ref().event_name,
                timestamp_to_string(data_received.get_ref().timestamp.clone()),
                data_received.get_ref().number.to_string()
            ),
        }))
    }
}

// implementing rpc for request to the service defined in consume.proto
#[tonic::async_trait]
impl ConsumeFromBroker for BrokerServer {
    // our rpc impelemented as function
    async fn send(
        &self,
        data_received: Request<ConsumeDataFromBroker>,
    ) -> Result<Response<BrokerToConsumerAck>, Status> {
        // returning a response as BrokerToConsumerAck message as defined in .proto
        
        let mut index_table_file = self.index_table.lock().unwrap();
        let mut events_log_file = self.events_log.lock().unwrap();
        let event_num = &(data_received.get_ref().number as u64);
        let index_length = index_table_file.seek(SeekFrom::End(0))? / 1; // length in bytes

        let metadata = fs::metadata("events.log")?;
        let file_size = metadata.len();

        println!("{} {}", file_size, index_length);

        index_table_file.seek(SeekFrom::Start(event_num*8))?;

        let curr_event_index;
        let next_event_index;
        if *event_num == (index_length / 8) - 1 {
            println!("Requesting last event");

            // set cur and next index
            let mut half_index_buffer = [0;8];
            index_table_file.read_exact(&mut half_index_buffer).expect("Couldn't read index table file\n");

            curr_event_index = usize::from_le_bytes(half_index_buffer.try_into().unwrap());
            next_event_index = file_size as usize;
        } else {
            println!("Requesting not-last event");
            
            let mut index_buffer = [0;16];
            index_table_file.read_exact(&mut index_buffer).expect("Couldn't read index table file\n");
            let (first_bytes, second_bytes) = index_buffer.split_at_mut(8);

            println!("{:?} {:?}", first_bytes, second_bytes);

            curr_event_index = usize::from_le_bytes(first_bytes.try_into().unwrap());
            next_event_index = usize::from_le_bytes(second_bytes.try_into().unwrap());
        }

        println!("{} {}", curr_event_index, next_event_index);

        let length_of_event = next_event_index - curr_event_index;
        let mut event_buffer = vec![0; length_of_event];

        events_log_file.seek(SeekFrom::Start(curr_event_index as u64))?;
        events_log_file.read_exact(&mut event_buffer).expect("Could not read event\n");

        let event_string = String::from_utf8(event_buffer).unwrap_or_else(|_| String::new());
        println!("Message content is {}", event_string);

        println!(
            "Received message from consumer: {}\nWith key number: {}\nWith content: {}",
            data_received.get_ref().event_name,
            data_received.get_ref().number.to_string(),
            event_string
        );
        let response_event = Event {
            event_name: String::from(event_string),
            timestamp: Some(Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
        };
        Ok(Response::new(BrokerToConsumerAck {
            event_vec: vec![response_event],
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our server
    let ip_port_string = format!("[::1]:50051");
    let addr = ip_port_string.parse().unwrap();

    // get env variables
    dotenv::dotenv().ok();
    let coord_ip = dotenv::var("COORD_IP").unwrap();
    let coord_port = dotenv::var("COORD_PORT").unwrap();
    let coord_address = format!("{}:{}", coord_ip, coord_port);
    println!("Coordinator is at {}", coord_address);

    // we have to define a service for each of our rpcs
    // andrew, these will soon be the bane of your existence (concurrency)
    let service1 = BrokerServer::new();
    let service2 = BrokerServer::new();

    // // creating a channel ie connection to server
    // let group_coordinator = tonic::transport::Channel::from_static("http://{}", coord_address)
    //     .connect()
    //     .await?;
    // // creating gRPC client from channel
    // let mut connection_to_gc = KafkaBrokerInitializationServiceClient::new(group_coordinator);

    // // creating a new Request to send to broker
    // let data_for_gc = tonic::Request::new(BrokerInitializationRequest {
    //     broker: /*I have NO IDEA what goes here*/,
    //     partition: 2,
    // });

    // let response_from_gc = connection_to_gc.send(data_for_gc).await?.into_inner();

    println!("Server listening on port {}", addr);
    // adding services to server and serving
    Server::builder()
        .add_service(PublishToBrokerServer::new(service1))
        .add_service(ConsumeFromBrokerServer::new(service2))
        .serve(addr)
        .await?;
    Ok(())
}
