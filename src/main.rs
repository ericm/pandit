#![feature(destructuring_assignment)]
#![feature(ptr_to_from_bits)]
#![feature(binary_heap_into_iter_sorted)]

pub mod api;
pub mod broker;
pub mod handlers;
pub mod proto;
pub mod server;
pub mod services;
pub mod writers;

use api_proto::api_grpc::create_api;
use clap;
use grpcio::ChannelBuilder;
use grpcio::Environment;
use grpcio::ResourceQuota;
use grpcio::ServerBuilder;
use std::convert::TryInto;
use std::sync::Arc;
use tokio;
use tokio::signal::ctrl_c;
use tokio::sync::RwLock;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

use crate::api::ApiServer;
use crate::broker::Broker;
use crate::server::IntraServer;
use crate::server::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let app = new_app();

    let cfg = services::new_config(app.get_matches().value_of("config").unwrap());
    let broker = Broker::connect(cfg.clone()).unwrap();
    let broker = Arc::new(RwLock::new(broker));

    let intra_server = {
        let server = IntraServer::default();
        let server = Arc::new(RwLock::new(server));
        let addr = cfg
            .get_str("server.address")
            .unwrap_or("localhost:50122".to_string());
        {
            let server = server.clone();
            tokio::spawn(async move {
                let server = server.read().await;
                server.run(addr).await.unwrap();
            });
        }
        server
    };

    let api_service = create_api(ApiServer::new(broker.clone(), intra_server.clone()));

    let env = Arc::new(Environment::new(1));
    let quota = ResourceQuota::new(Some("ApiServerQuota")).resize_memory(1024 * 1024);
    let ch_builder = ChannelBuilder::new(env.clone()).set_resource_quota(quota);
    let mut server = ServerBuilder::new(env)
        .register_service(api_service)
        .bind(
            "127.0.0.1",
            cfg.get_int("admin.port")
                .unwrap_or(50121)
                .try_into()
                .unwrap(),
        )
        .channel_args(ch_builder.build_args())
        .build()
        .unwrap();
    server.start();

    let iface = "lo";
    println!("Attaching socket to interface {}", iface);

    let ctrlc_fut = async {
        ctrl_c().await.unwrap();
    };
    println!("Hit Ctrl-C to quit");
    ctrlc_fut.await;
}

fn new_app() -> clap::App<'static, 'static> {
    clap::App::new("pandit")
        .version("1.0")
        .author("Eric Moynihan")
        .about("Pandit CLI")
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .default_value("./.pandit.yml"),
        )
}
