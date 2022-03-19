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
use console::style;
use dashmap::DashMap;
use get_if_addrs::get_if_addrs;
use grpcio::ChannelBuilder;
use grpcio::EnvBuilder;
use grpcio::Environment;
use grpcio::ResourceQuota;
use grpcio::ServerBuilder;
use log::LevelFilter;
use log::Metadata;
use log::Record;
use std::convert::TryInto;
use std::env::current_dir;
use std::fs::read_dir;
use std::str::FromStr;
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

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &Record) {
        match record.level() {
            log::Level::Error => eprintln!(
                "{}: {}",
                style(record.level()).red().bold(),
                style(record.args()).bright().bold()
            ),
            log::Level::Warn => eprintln!(
                "{}: {}",
                style(record.level()).yellow().bold(),
                style(record.args()).bright().bold()
            ),
            _ => println!(
                "{}: {}",
                style(record.level()).green().bold(),
                style(record.args()).bright().bold()
            ),
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

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
    #[clap(long, default_value = "INFO")]
    level: String,
    #[clap(long)]
    docker: bool,
    #[clap(long)]
    k8s: bool,
    #[clap(long)]
    tokio_tracing: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app: Args = Parser::parse();
    if app.tokio_tracing {
        console_subscriber::init();
    } else {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::from_str(app.level.as_str()).unwrap())
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Info))
            .unwrap();
    }

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
        let k = Arc::new(K8sHandler::new(admin_port).await.unwrap());
        {
            let k = k.clone();
            tokio::spawn(async move {
                k.run().await;
            });
        }
        log::info!("Started in K8s mode");
        Some(k)
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
    start_services(
        &cfg,
        k8s_handler.clone(),
        broker.clone(),
        intra_server.clone(),
    )
    .await;

    tokio::spawn(async move {
        loop {
            match broker.receive(k8s_handler.clone()).await {
                Ok(_) => {}
                Err(err) => log::error!("error interfacing with broker: {:?}", err),
            }
        }
    });

    log::info!("Hit Ctrl-C to quit");
    server_cancelled.await.unwrap();
}

async fn start_services(
    cfg: &config::Config,
    k8s_handler: Option<Arc<K8sHandler>>,
    broker: Arc<Broker>,
    server: Arc<RwLock<IntraServer>>,
) {
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
        Some(_) => {
            let pods = Arc::new(pods);
            tokio::spawn(async move { broker.watch_pods(pods, server).await.unwrap() });
        }
        None => {}
    }
}
