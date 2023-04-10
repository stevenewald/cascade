mod publish {
    tonic::include_proto!("publish");
}
use publish::publish_client::PublishClient;
use publish::PublishRequest;
use prost_types::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // creating a channel ie connection to server
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;
    // creating gRPC client from channel
    let mut client = PublishClient::new(channel);

    // getting the current time as a Duration since UNIX_EPOCH
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");

    // converting Duration to Timestamp
    let timestamp = Timestamp {
        seconds: now.as_secs() as i64,
        nanos: now.subsec_nanos() as i32,
    };

    // creating a new Request
    let request = tonic::Request::new(PublishRequest {
        event: String::from("first"),
        timestamp: Some(timestamp),
    });
    // sending request and waiting for response
    let response = client.send(request).await?.into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}
