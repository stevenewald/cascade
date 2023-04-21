mod publish {
    //this compiles the publish.proto file and generates a rust code for the gRPC services
    //we then import this rust code below
    tonic::include_proto!("publish");
}

mod consume {
    tonic::include_proto!("consume");
}
use consume::consume_from_broker_server::{ConsumeFromBroker, ConsumeFromBrokerServer};
use consume::{BrokerToConsumerAck, ConsumeDataFromBroker, Pair};
use publish::publish_to_broker_server::{PublishToBroker, PublishToBrokerServer};
use publish::{BrokerToPublisherAck, PublishDataToBroker};

use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;
use tonic::{transport::Server, Request, Response, Status};

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
        println!(
            "Received message from consumer: {}\nWith key number: {}",
            data_received.get_ref().event_name,
            data_received.get_ref().number.to_string()
        );
        let response_pair = Pair {
            event_name: String::from("event 1"),
            timestamp: Some(Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
        };
        Ok(Response::new(BrokerToConsumerAck {
            pair_vec: vec![response_pair],
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our server
    let addr = "[::1]:50051".parse().unwrap();

    // we have to define a service for each of our rpcs
    //andrew, these will soon be the bane of your existence (concurrency)
    let service1 = BrokerServer::default();
    let service2 = BrokerServer::default();
    println!("Server listening on port {}", addr);
    // adding services to server and serving
    Server::builder()
        .add_service(PublishToBrokerServer::new(service1))
        .add_service(ConsumeFromBrokerServer::new(service2))
        .serve(addr)
        .await?;
    Ok(())
}
