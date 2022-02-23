use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};
fn main() {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("localhost:50051");
    let client = api_proto::api_grpc::ApiClient::new(ch);

    let app = new_app();
}

fn new_app() -> clap::App<'static, 'static> {
    clap::App::new("pandit")
        .version("1.0")
        .author("Eric Moynihan")
        .about("Pandit CLI")
        .subcommand(
            clap::SubCommand::with_name("service").subcommand(clap::SubCommand::with_name("add")),
        )
}
