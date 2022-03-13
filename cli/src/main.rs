use clap::{Parser, Subcommand};
use config::Config;
use console::{style, Emoji};
use grpcio::{ChannelBuilder, EnvBuilder};
use indicatif::ProgressStyle;
use serde::Deserialize;
use std::{env::current_dir, fs::File, io::Read, path::PathBuf, sync::Arc, time::Duration};
use tokio;

#[derive(Parser)]
#[clap(author = "Eric Moynihan", version, about, name = "pandit", long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(short, long, default_value = "/etc/pandit/protos")]
    proto_path: String,
    #[clap(short, long, default_value = "localhost:50121")]
    daemon_address: String,
    #[clap(subcommand)]
    service: ServiceCommand,
}

#[derive(Subcommand)]
enum ServiceCommand {
    #[clap(about = "Add a new service to pandit")]
    Add {
        #[clap(help = "Path to the panditfile. If a directory, will look for ./panditfile.toml")]
        path: String,
    },
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Panditfile {
    metadata: Metadata,
    docker: Option<Docker>,
    k8s: Option<K8s>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Metadata {
    name: String,
    port: i32,
    proto: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Docker {
    container_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(unused)]
enum K8s {
    Pod { name: String },
    Service { name: String },
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // std::panic::set_hook(Box::new(|v| {
    //     log::error!(
    //         "      {}An error occurred in '{}'...",
    //         Emoji("❌ ", ""),
    //         v.to_string(),
    //     );
    // }));
    let app: Args = Parser::parse();
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(app.daemon_address.as_str());
    let client = api_proto::api_grpc::ApiClient::new(ch);
    let proto_path = {
        let mut path = PathBuf::from(app.proto_path);
        if path.is_relative() {
            path = current_dir().unwrap().join(path);
        }
        path.canonicalize().unwrap()
    };

    println!(
        "{} {}Using proto library '{}'...",
        style("[1/3]").bold().dim(),
        Emoji("🔍 ", ""),
        style(proto_path.to_str().unwrap()).green(),
    );

    match &app.service {
        ServiceCommand::Add { path } => {
            let path = PathBuf::from(path);
            let mut path = path.canonicalize().unwrap();
            if !path.ends_with(".toml") {
                path = path.join("panditfile.toml");
            }
            let cfg: Panditfile = {
                let cfg = Config::builder()
                    .add_source(config::File::with_name(path.to_str().unwrap()))
                    .build()
                    .unwrap();
                cfg.try_deserialize().unwrap()
            };
            println!(
                "{} {}Using panditfile '{}'...",
                style("[2/3]").bold().dim(),
                Emoji("📃 ", ""),
                style(path.to_str().unwrap()).green()
            );

            let mut proto_path = proto_path.join(&cfg.metadata.proto);
            proto_path.set_extension("proto");

            let mut proto = Vec::<u8>::new();
            let mut panditfile = File::open(&proto_path).unwrap();
            panditfile.read_to_end(&mut proto).unwrap();

            let mut req = api_proto::api::StartServiceRequest::new();
            req.set_proto(proto);
            req.set_port(cfg.metadata.port);
            req.set_name(cfg.metadata.name.clone());
            match cfg.docker {
                Some(docker) => {
                    req.set_docker_id(docker.container_id);
                }
                None => match cfg.k8s {
                    Some(k8s) => match k8s {
                        K8s::Pod { name } => req.set_k8s_pod(name),
                        K8s::Service { name } => req.set_k8s_service(name),
                    },
                    None => {}
                },
            }

            let pb = indicatif::ProgressBar::new_spinner();
            pb.set_message("Awaiting response from pandit...");
            pb.enable_steady_tick(20);
            let _resp = client.start_service(&req).unwrap();
            pb.finish_and_clear();
            println!(
                "{} {}Successfully created service '{}' with proto '{}'...",
                style("[3/3]").bold().dim(),
                Emoji("✅ ", ""),
                style(cfg.metadata.name).green(),
                style(cfg.metadata.proto).green()
            );
        }
    }
}
