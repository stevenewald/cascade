mod coordinate {
    //this compiles the coordinator.proto file and generates a rust code for the gRPC services
    //we then import this rust code below
    tonic::include_proto!("coordinate");
}

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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // the port 50001 result in connection error
    // creating a channel ie connection to server
    let coordinator_channel = tonic::transport::Channel::from_static("http://[::1]:50001")
        .connect()
        .await?;
    // creating gRPC client for KafkaMetadataService from channel
    let mut kafka_metadata_service_client = KafkaMetadataServiceClient::new(coordinator_channel);
    // creating a channel ie connection to server
    // let broker_channel = tonic::transport::Channel::from_static("http://[::1]:50051")
    //     .connect()
    //     .await?;
    // // creating gRPC client from channel
    // let mut client_connection_to_broker = PublishToBrokerClient::new(broker_channel);

    // creating a new Request to send to KafkaMetadataService
    let metadata_request = tonic::Request::new(MetadataRequest {
        topic_names: vec![String::from("default")],
    });
    // sending metadata_request and waiting for response
    let metadata_response = kafka_metadata_service_client.get_metadata(metadata_request).await?.into_inner();
    let num_partitions = metadata_response.brokers.len();
    println!("Received metadata from coordinator with {} partitions.", num_partitions);

    // events_to_send (made up) = [a, b, c, d]
    // brokers = [1, 2, 3, 4]
    // counter = 0
    // while counter < len(events_to_send):
        // brokers[counter % len(brokers)].send(events_to_send[counter])
        // counter+=1




    //starting here, everything will be in a for loop (loop through brokers in response)
    // getting the current time as a Duration since UNIX_EPOCH
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");

    // converting Duration to Timestamp
    let timestamp = Timestamp {
        seconds: now.as_secs() as i64,
        nanos: now.subsec_nanos() as i32,
    };

    // creating a new Request to send to broker
    let mut rng = rand::thread_rng();
    let data_to_broker = tonic::Request::new(PublishDataToBroker {
        event_name: String::from("default"),
        timestamp: Some(timestamp),
        number: rng.gen::<i32>(), // this is where the cpu usage (%) will go, make it a float though
    });
    // sending data_to_broker and waiting for response
    let ack_from_broker = client_connection_to_broker.send(data_to_broker).await?.into_inner();
    println!("Received acknowledgement from broker with message: {}", ack_from_broker.response_to_producer);

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

