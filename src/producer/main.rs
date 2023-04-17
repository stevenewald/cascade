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
    // creating a channel ie connection to server
    let channel = tonic::transport::Channel::from_static("http://[::1]:50052")
        .connect()
        .await?;
    // creating gRPC client from channel
    let mut client_connection_to_broker = PublishToBrokerClient::new(channel);

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
        number: rng.gen::<i32>(), // set the number field to a random integer

    });
    // sending data_to_broker and waiting for response
    let ack_from_broker = client_connection_to_broker.send(data_to_broker).await?.into_inner();
    println!("RESPONSE={:?}", ack_from_broker);
    Ok(())
}


