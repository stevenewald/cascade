mod coordinate {
    tonic::include_proto!("coordinate");
}

use std::sync::{Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use coordinate::Broker;

pub struct BrokerMap(RwLock<HashMap<String, Mutex<HashSet<Broker>>>>);

impl std::hash::Hash for Broker {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::cmp::Eq for Broker {}

impl BrokerMap {
    fn new() -> Self {
        BrokerMap(RwLock::new(HashMap::new()))
    }

    fn insert(&self, topic: String, broker: Broker) -> Result<(), Box<dyn std::error::Error>> {
        let mut map = self.0.write().unwrap();

        if let Some(mutex_set) = map.get(&topic as &str) {
            let mut set = mutex_set.lock().unwrap();
            set.insert(broker);
        } else {
            let mut set = HashSet::new();
            set.insert(broker);
            map.insert(topic, Mutex::new(set));
        }

        Ok(())
    }

    fn remove(&self, topic: &String, broker: &Broker) -> Result<(), Box<dyn std::error::Error>> {
        let mut map = self.0.write().unwrap();
        
        if let Some(mutex_set) = map.get_mut(&topic as &str) {
            let mut set = mutex_set.lock().unwrap();
            if !set.remove(broker) {
                return Err("Broker not found".into());
            }
        } else {
            return Err("Topic not found".into());
        }

        Ok(())
    }

    fn get_topic_brokers(&self, topic: &String) -> Result<HashSet<Broker>, Box<dyn std::error::Error>> {
        let map = self.0.read().unwrap();

        if let Some(mutex_set) = map.get(&topic as &str) {
            let set = mutex_set.lock().unwrap();
            return Ok(set.clone());  
        }

        Err("Topic not found".into())
    }
}

