use tonic;
use tonic::transport;

pub async fn send(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let channel = transport::Channel::from_shared(addr)
        .unwrap()
        .connect()
        .await?;
}
