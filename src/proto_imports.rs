pub mod coordinate {
    tonic::include_proto!("coordinate");
}

use coordinate::Broker;
impl std::hash::Hash for Broker {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::cmp::Eq for Broker {}
