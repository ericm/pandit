use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};

mod example1;
mod example1_grpc;

fn main() {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("localhost:50122");
    let client = example1_grpc::ExampleServiceClient::new(ch);

    let mut req = example1::ExampleRequest::new();
    req.set_id(1);
    let resp = client.get_example(&req).unwrap();
    println!("{} {}", resp.id, resp.user);
}
