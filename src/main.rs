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
use bollard::Docker;
use clap::Parser;
use grpcio::ChannelBuilder;
use grpcio::EnvBuilder;
use grpcio::Environment;
use grpcio::ResourceQuota;
use grpcio::ServerBuilder;
use std::convert::TryInto;
use std::env::current_dir;
use std::fs::read_dir;
use std::fs::File;
use std::sync::Arc;
use tokio;
use tokio::signal::ctrl_c;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

use crate::api::ApiServer;
use crate::api::DockerNetworkRuntime;
use crate::api::NetworkRuntime;
use crate::broker::Broker;
use crate::server::IntraServer;
use crate::server::Server;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(short, long, default_value = "./.pandit.yml")]
    config: String,
    #[clap(short, long)]
    docker: bool,
    #[clap(short, long)]
    k8s: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::ERROR)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let app: Args = Parser::parse();

    let network_runtime: Option<Arc<dyn NetworkRuntime>> = if app.docker {
        let docker = Docker::connect_with_socket_defaults().unwrap();
        let client = Arc::new(DockerNetworkRuntime::new(docker));
        {
            let client = client.clone();
            tokio::spawn(async move {
                client.run().await;
            });
        }
        Some(client)
    } else if app.k8s {
        // let k8s =
        None
    } else {
        None
    };

    let cfg = services::new_config(app.config.as_str());
    let broker = Broker::connect(cfg.clone()).unwrap();
    let broker = Arc::new(broker);

    let server_cancelled: JoinHandle<()>;
    let intra_server = {
        let server = IntraServer::new(broker);
        let server = Arc::new(RwLock::new(server));
        let addr = cfg
            .get_str("server.address")
            .unwrap_or("0.0.0.0:50122".to_string());
        {
            let server = server.clone();

            server_cancelled = tokio::spawn(async move {
                let server = server.read().await;
                server.run(addr).await.unwrap();
            });
        }
        server
    };

    let api_service = create_api(ApiServer::new(
        broker.clone(),
        intra_server.clone(),
        network_runtime,
    ));

    let env = Arc::new(Environment::new(1));
    let quota = ResourceQuota::new(Some("ApiServerQuota")).resize_memory(1024 * 1024);
    let ch_builder = ChannelBuilder::new(env.clone()).set_resource_quota(quota);
    let mut server = ServerBuilder::new(env)
        .register_service(api_service)
        .bind(
            "0.0.0.0",
            cfg.get_int("admin.port")
                .unwrap_or(50121)
                .try_into()
                .unwrap(),
        )
        .channel_args(ch_builder.build_args())
        .build()
        .unwrap();
    server.start();
    start_services(&cfg);

    tokio::spawn(async move {
        loop {
            match broker.receive().await {
                Ok(_) => {}
                Err(err) => eprintln!("error interfacing with broker: {:?}", err),
            }
        }
    });

    println!("Hit Ctrl-C to quit");
    server_cancelled.await.unwrap();
}

fn start_services(cfg: &config::Config) {
    let addr = format!("0.0.0.0:{}", cfg.get_int("admin.port").unwrap_or(50121));
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(addr.as_str());
    let client = api_proto::api_grpc::ApiClient::new(ch);

    let paths = read_dir(current_dir().unwrap()).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let ext = match path.extension() {
            Some(ext) => ext.to_str().unwrap(),
            None => continue,
        };
        if ext != "pandit_service" {
            continue;
        }
        let save_file = File::open(path).unwrap();
        let save: serde_json::Value = serde_json::from_reader(save_file).unwrap();
        {
            use std::convert::TryFrom;
            let name = save.get("name").unwrap().as_str().unwrap().to_string();
            let port: i32 = save
                .get("port")
                .unwrap()
                .as_i64()
                .unwrap()
                .try_into()
                .unwrap();
            let proto: Vec<u8> = save
                .get("proto")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
                .collect();
            let container_id = save
                .get("container_id")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let mut req = api_proto::api::StartServiceRequest::new();
            req.set_proto(proto);
            req.set_port(port);
            req.set_name(name);
            req.set_container_id(container_id);
            client.start_service(&req).unwrap();
        }
    }
}
