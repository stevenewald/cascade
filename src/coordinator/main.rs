mod coordinate {
    //this compiles the publish.proto file and generates a rust code for the gRPC services
    //we then import this rust code below
    tonic::include_proto!("coordinate");
}

use coordinate::broker_initialization_response::StatusCode;
use coordinate::kafka_metadata_service_server::{KafkaMetadataService, KafkaMetadataServiceServer};
use coordinate::kafka_broker_initialization_service_server::{KafkaBrokerInitializationService, KafkaBrokerInitializationServiceServer};

use coordinate::{Broker, MetadataRequest, MetadataResponse, BrokerInitializationRequest, BrokerInitializationResponse};

use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;
use tonic::{transport::Server, Request, Response, Status};

use std::sync::Mutex;
use std::io::{Read, Write, Seek, SeekFrom};
use std::fs::OpenOptions;
use std::collections::{HashMap, HashSet};

impl std::hash::Hash for Broker {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::cmp::Eq for Broker {}

// impl std::cmp::PartialEq for Broker {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id && self.ip == other.ip && self.port == other.port;
//     }
// }

struct CoordinatorServer {
    // Map between topic and a set of brokers who hold the topic
    broker_metadata: Mutex<HashMap<String, Mutex<HashSet<Broker>>>>
}

impl CoordinatorServer {
    fn new() -> Self {
        let broker_metadata: HashMap<String, Mutex<HashSet<Broker>>> = HashMap::new();
        println!("opened file!");

        CoordinatorServer {
            broker_metadata: Mutex::new(broker_metadata)
        }
    }
}

#[tonic::async_trait]
impl KafkaBrokerInitializationService for CoordinatorServer {
    async fn initialize_broker(
        &self,
        data_received: Request<BrokerInitializationRequest>,
    ) -> Result<Response<BrokerInitializationResponse>, Status> {
        let broker: Broker = data_received.get_ref().broker.clone().unwrap();
        let partition: u32 = data_received.get_ref().partition;
        let topic_name: &String = &data_received.get_ref().topic_name;

        let mut metadata_map = self.broker_metadata.lock().unwrap();

        if metadata_map.contains_key(topic_name) {
            let mut topic_brokers = metadata_map.get(topic_name).unwrap().lock().unwrap();
            if !topic_brokers.contains(&broker) {
                topic_brokers.insert(broker);
            } else {
                let resp = Response::new(BrokerInitializationResponse {
                    status: 1,
                    message: "Broker already registered".to_string(),
                });
                Ok::<tonic::Response<BrokerInitializationResponse>,u8>(resp);
            }
        } else {
            metadata_map.insert(topic_name.to_string(), Mutex::new(HashSet::new()));
            metadata_map.get(topic_name).unwrap().lock().unwrap().insert(broker);
        }

        drop(metadata_map);

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
        let topic_name: &String = &data_received.get_ref().topic_name;
        let mut metadata_map = self.broker_metadata.lock().unwrap();
        let brokers: Vec<Broker> = Vec::new();

        if metadata_map.contains_key(topic_name) {
            let mut topic_brokers = metadata_map.get(topic_name).unwrap().lock().unwrap();
            for broker in topic_brokers.iter() {
                brokers.push(broker.clone());
            }
        }
        
        drop(metadata_map);

        Ok(Response::new(MetadataResponse {
            brokers: brokers,
        }))
    }
}

fn main() {

}

