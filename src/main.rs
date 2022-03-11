#![feature(ptr_to_from_bits)]
#![feature(binary_heap_into_iter_sorted)]

pub mod api;
pub mod broker;
pub mod handlers;
pub mod proto;
pub mod server;
pub mod services;
pub mod writers;

use api::add_service_from_file;
use api_proto::api_grpc::create_api;
use bollard::Docker;
use clap::Parser;
use dashmap::DashMap;
use get_if_addrs::get_if_addrs;
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
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::api::ApiServer;
use crate::api::DockerNetworkRuntime;
use crate::api::K8sHandler;
use crate::api::NetworkRuntime;
use crate::broker::Broker;
use crate::server::IntraServer;
use crate::server::Server;

#[derive(Parser)]
#[clap(name = "panditd", author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(short, long, default_value = "./.pandit.yml")]
    config: String,
    #[clap(short, long)]
    interface: Option<String>,
    #[clap(short, long)]
    address: Option<String>,
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
    let cfg = services::new_config(app.config.as_str());

    if app.k8s && app.docker {
        panic!("error: cannot use both --k8s and --docker");
    }

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
    } else {
        None
    };
    let k8s_handler = if app.k8s {
        let admin_port: u16 = cfg
            .get_int("admin.port")
            .unwrap_or(50121)
            .try_into()
            .unwrap();
        Some(K8sHandler::new(admin_port).await.unwrap())
    } else {
        None
    };

    let address = match app.address {
        Some(v) => v,
        None => match app.interface {
            Some(interface) => {
                let mut ip = None;
                for iface in get_if_addrs().unwrap() {
                    if iface.name == interface {
                        ip = Some(iface.addr.ip().to_string());
                        break;
                    }
                }
                match ip {
                    Some(ip) => ip,
                    None => "0.0.0.0".to_string(),
                }
            }
            None => "0.0.0.0".to_string(),
        },
    };

    let broker = Broker::connect(cfg.clone(), address.clone()).unwrap();
    let broker = Arc::new(broker);

    let server_cancelled: JoinHandle<()>;
    let intra_server = {
        let server = IntraServer::new(broker.clone());
        let server = Arc::new(RwLock::new(server));
        let addr = cfg
            .get_str("server.address")
            .unwrap_or(format!("{}:50122", address));
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
        k8s_handler.clone(),
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
    match k8s_handler.clone() {
        Some(mut handler) => {
            handler.add_server_broker(broker.clone(), intra_server.clone());
            start_services(&cfg, Some(handler)).await;
        }
        None => {
            start_services(&cfg, None).await;
        }
    }

    tokio::spawn(async move {
        loop {
            match broker.receive(k8s_handler.clone()).await {
                Ok(_) => {}
                Err(err) => eprintln!("error interfacing with broker: {:?}", err),
            }
        }
    });

    println!("Hit Ctrl-C to quit");
    server_cancelled.await.unwrap();
}

async fn start_services(cfg: &config::Config, k8s_handler: Option<K8sHandler>) {
    let addr = format!("0.0.0.0:{}", cfg.get_int("admin.port").unwrap_or(50121));
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(addr.as_str());
    let client = api_proto::api_grpc::ApiClient::new(ch);
    let paths = read_dir(current_dir().unwrap()).unwrap();

    let mut pods = DashMap::new();
    for path in paths {
        let path = path.unwrap().path();
        add_service_from_file(path, &k8s_handler, &mut pods, &client)
            .await
            .unwrap();
    }
    match k8s_handler {
        Some(handler) => {
            let pods = Arc::new(pods);
            let handler = Arc::new(handler);
            tokio::spawn(async move { handler.watch_pods(pods).await.unwrap() });
        }
        None => {}
    }
}
