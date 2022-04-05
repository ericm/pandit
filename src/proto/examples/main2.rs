use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};

mod example2;
mod example2_grpc;

fn main() {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("localhost:50122");
    let client = example2_grpc::PostgreSqlClient::new(ch);

    let mut req = example2::ExampleRequest::new();
    req.set_id(1);
    req.set_user("test".into());
    let resp = client.get_example(&req).unwrap();
    println!("{} {}", resp.id, resp.user);
}

