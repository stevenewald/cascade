mod publish {
    tonic::include_proto!("publish");
}
use publish::publish_server::{Publish, PublishServer};
use publish::{PublishRequest, PublishResponse};
use tonic::{transport::Server, Request, Response, Status};
use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;

fn timestamp_to_string(timestamp: Option<Timestamp>) -> String {
    if let Some(ts) = timestamp {
        if let Some(naive_datetime) = NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32) {
            let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
            datetime.to_rfc3339()
        } else {
            "Invalid timestamp".to_string()
        }
    } else {
        "No timestamp provided".to_string()
    }
}

// defining a struct for our service
#[derive(Default)]
pub struct MyPublish {}

// implementing rpc for service defined in .proto
#[tonic::async_trait]
impl Publish for MyPublish {
    // our rpc impelemented as function
    async fn send(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        // returning a response as PublishResponse message as defined in .proto
        println!("Received message: {}", request.get_ref().event);
        Ok(Response::new(PublishResponse {
            // reading data from request which is awrapper around our PublishRequest message defined in .proto
            response: format!("publish: {}, {}", request.get_ref().event, timestamp_to_string(request.get_ref().timestamp.clone())),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = "[::1]:50051".parse().unwrap();
    // creating a service
    let say = MyPublish::default();
    println!("Server listening on {}", addr);
    // adding our service to our server.
    Server::builder()
        .add_service(PublishServer::new(say))
        .serve(addr)
        .await?;
    Ok(())
}
