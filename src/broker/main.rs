mod publish {
    //this compiles the publish.proto file and generates a rust code for the gRPC services
    //we then import this rust code below
    tonic::include_proto!("publishi");
}
use publish::publish_to_broker_server::{PublishToBroker, PublishToBrokerServer};
use publish::{PublishDataToBroker, BrokerToPublisherAck};

use tonic::{transport::Server, Request, Response, Status};
use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;

//this function is just to print and parse the timestamp of the event we receive
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

// defining a struct for our publish to broker service (this side, the broker, is receiving and replying)
#[derive(Default)]
pub struct BrokerServer {}

// implementing rpc for publish to the service defined in publish.proto
#[tonic::async_trait]
impl PublishToBroker for BrokerServer {
    // our rpc impelemented as function
    async fn send(
        &self,
        data_received: Request<PublishDataToBroker>,
    ) -> Result<Response<BrokerToPublisherAck>, Status> {
        // returning a response as BrokerToPublisherAck message as defined in .proto
        println!("Received message: {}", data_received.get_ref().event_name);
        Ok(Response::new(BrokerToPublisherAck {
            // reading data from request which is awrapper around our PublishDataToBroker message defined in .proto
            response_to_producer: format!("Broker response: received event with name {} and timestamp {} and number {}", data_received.get_ref().event_name, timestamp_to_string(data_received.get_ref().timestamp.clone()), data_received.get_ref().number.to_string()),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = "[::1]:50051".parse().unwrap();
    // creating a service
    let server = BrokerServer::default();
    println!("Server listening on {}", addr);
    // adding our service to our server.
    Server::builder()
        .add_service(PublishToBrokerServer::new(server))
        .serve(addr)
        .await?;
    Ok(())
}
