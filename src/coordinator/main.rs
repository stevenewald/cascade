mod brokermap;
use lazy_static::lazy_static;
// mod crate::proto_imports;
use kafka_clone::proto_imports::coordinate as coordinate;
use coordinate::kafka_metadata_service_server::{KafkaMetadataService, KafkaMetadataServiceServer};
use coordinate::kafka_broker_initialization_service_server::{KafkaBrokerInitializationService, KafkaBrokerInitializationServiceServer};

use coordinate::{Broker, MetadataRequest, MetadataResponse, BrokerInitializationRequest, BrokerInitializationResponse};

use tonic::{transport::Server, Request, Response, Status};

use brokermap::{BrokerMap};

use std::sync::Arc;

lazy_static! {
    static ref BROKER_METADATA_DICT: Arc<BrokerMap> = Arc::new(BrokerMap::new());
}


struct CoordinatorServer {
    // Map between topic and a set of brokers who hold the topic
    broker_metadata: Arc<BrokerMap> 
}

impl CoordinatorServer {
    fn new(broker_dict: Arc<BrokerMap>) -> Self {
        // let broker_metadata: HashMap<String, Mutex<HashSet<Broker>>> = HashMap::new();

        CoordinatorServer {
            broker_metadata: broker_dict
        }
    }
}

#[tonic::async_trait]
impl KafkaBrokerInitializationService for CoordinatorServer {
    async fn send(
        &self,
        data_received: Request<BrokerInitializationRequest>,
    ) -> Result<Response<BrokerInitializationResponse>, Status> {
        let broker: Broker = data_received.get_ref().broker.clone().unwrap();
        // TODO: partition is part of initialization request, but is not part of 
        // metdata request. we should probably be mapping (broker, parition) -> topic_name
        // let partition: u32 = data_received.get_ref().partition;
        let topic_name: &String = &data_received.get_ref().topic_name;

        if !self.broker_metadata.insert(topic_name.to_owned(), broker).unwrap() {
            let resp = Response::new(BrokerInitializationResponse {
                status: 1,
                message: "Broker already registered".to_string(),
            });
            return Ok::<tonic::Response<BrokerInitializationResponse>,Status>(resp);
        }

        println!("Broker initialized");

        Ok(Response::new(BrokerInitializationResponse {
            status: 0,
            message: "Broker successfully registered".to_string()
        }))
    }
}

#[tonic::async_trait]
impl KafkaMetadataService for CoordinatorServer {
    async fn get_metadata(
        &self,
        data_received: Request<MetadataRequest>,
    ) -> Result<Response<MetadataResponse>, Status> {
        println!("Received request for metadata");
        let topic_name: &String = &data_received.get_ref().topic_name;

        let topic_brokers = self.broker_metadata.get_topic_brokers(topic_name).unwrap();
        let brokers = topic_brokers.iter().cloned().collect();
        

        Ok(Response::new(MetadataResponse {
            brokers: brokers,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our server
    let addr = "0.0.0.0:50051".parse().unwrap();

    // we have to define a service for each of our rpcs
    // let broker_metadata_dict = Box::new(BrokerMap::new());
    //andrew, these will soon be the bane of your existence (concurrency)
    let service1: CoordinatorServer = CoordinatorServer::new(Arc::clone(&BROKER_METADATA_DICT));
    let service2 = CoordinatorServer::new(Arc::clone(&BROKER_METADATA_DICT));
    println!("Coordinator listening on port {}", addr);
    // adding services to server and serving
    Server::builder()
        .add_service(KafkaBrokerInitializationServiceServer::new(service1))
        .add_service(KafkaMetadataServiceServer::new(service2))
        .serve(addr)
        .await?;
    Ok(())
}
