mod consume {
    //this compiles the consume.proto file and generates a rust code for the gRPC services
    //we then import this rust code below
    tonic::include_proto!("consume");
}
use consume::consume_from_broker_client::ConsumeFromBrokerClient;
use consume::ConsumeDataFromBroker;
use rand::Rng;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // creating a channel ie connection to server
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;
    // creating gRPC client from channel
    let mut client_connection_to_broker = ConsumeFromBrokerClient::new(channel);

    // creating a new Request to send to broker
    let mut rng = rand::thread_rng();
    let data_to_broker = tonic::Request::new(ConsumeDataFromBroker {
        event_name: String::from("req_from_consumer"),
        number: rng.gen::<u8>() as u32,
        //this casting is purely for testing bc smaller numbers are nicer to look at
    });

    // sending data_to_broker and waiting for response
    let ack_from_broker = client_connection_to_broker.send(data_to_broker).await?.into_inner();
    println!("Received {} events from broker, first event_string is {}", ack_from_broker.pair_vec.len(), ack_from_broker.pair_vec[0].event_name);

    Ok(())

}


