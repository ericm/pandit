use clap::{Parser, Subcommand};
use config::Config;
use console::{style, Emoji};
use grpcio::{ChannelBuilder, EnvBuilder};
use indicatif::ProgressStyle;
use serde::Deserialize;
use std::{
    env::current_dir, error::Error, ffi::OsStr, fs::File, io::Read, path::PathBuf, process::exit,
    str::FromStr, sync::Arc, time::Duration,
};
use tokio::{self, fs::create_dir_all};

mod packages;

#[derive(Parser)]
#[clap(author = "Eric Moynihan", version, about, name = "pandit", long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(short, long, default_value = "/etc/pandit/protos")]
    proto_path: String,
    #[clap(short, long, default_value = "localhost:50121")]
    daemon_address: String,
    #[clap(
        long,
        default_value = "https://raw.githubusercontent.com/ericm/pandit-packages/main/index.json"
    )]
    repo_index: String,
    #[clap(subcommand)]
    service: ServiceCommand,
}

#[derive(Debug, Subcommand, Clone)]
enum ServiceCommand {
    #[clap(about = "Add a new service to pandit")]
    Add {
        #[clap(help = "Path to the panditfile. If a directory, will look for ./panditfile.toml")]
        path: String,
    },
    #[clap(about = "Install a pandit package")]
    Install {
        #[clap(help = "Name of package to install")]
        name: String,
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
    let app: Args = Parser::parse();
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(app.daemon_address.as_str());
    let client = api_proto::api_grpc::ApiClient::new(ch);
    let proto_path = {
        let mut path = PathBuf::from(app.proto_path.clone());
        if path.is_relative() {
            path = current_dir().unwrap().join(path);
        }
        match path.canonicalize() {
            Ok(v) => v,
            Err(_) => {
                {
                    let path = path.clone();
                    create_dir_all(path).await.unwrap();
                }
                println!(
                    "      {}{}created empty proto directory...",
                    Emoji("⚠️ ", ""),
                    style("Warning: ").yellow().bold()
                );
                path.canonicalize().unwrap()
            }
        }
    };

    match app.service.clone() {
        ServiceCommand::Add { path } => {
            println!(
                "{} {}Using proto library '{}'...",
                style("[1/3]").bold().dim(),
                Emoji("🔍 ", ""),
                style(proto_path.to_str().unwrap()).green(),
            );
            let path = PathBuf::from(path);
            let mut path = path.canonicalize().unwrap();
            if path.extension() != Some(OsStr::new("toml")) {
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
        ServiceCommand::Install { name } => {
            println!(
                "{} {}Pulling index of packages...",
                style("[1/?]").bold().dim(),
                Emoji("⬇️  ", ""),
            );
            let index = match packages::Index::get(app.repo_index.clone()).await {
                Ok(v) => v,
                Err(e) => {
                    error("pulling index of packages", e);
                    exit(1);
                }
            };
            println!(
                "{} {}Searching for package '{}'...",
                style("[2/?]").bold().dim(),
                Emoji("🔍 ", ""),
                style(&name).green().bold(),
            );
            let pkg = match index.packages.get(&name) {
                Some(v) => v,
                None => {
                    error(
                        "finding package in index",
                        format!("no package called '{}'", name).into(),
                    );
                    exit(1);
                }
            };
            match pkg.install(name, &app.proto_path).await {
                Ok(_) => {}
                Err(err) => {
                    error("installing the package", err);
                    exit(1);
                }
            };
        }
    }
}

fn error(info: &str, err: Box<dyn Error>) {
    eprintln!(
        "      {}An error occurred {}: '{}'",
        Emoji("❌ ", ""),
        info,
        style(err).red().bold(),
    );
}
