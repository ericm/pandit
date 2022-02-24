use clap::{Parser, Subcommand};
use config::Config;
use grpcio::{ChannelBuilder, EnvBuilder};
use std::{env::current_dir, fs::File, io::Read, path::PathBuf, sync::Arc};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
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
    Add { path: String },
}

fn main() {
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
    println!("Using proto library: {}", proto_path.to_str().unwrap());

    match &app.service {
        ServiceCommand::Add { path } => {
            let mut req = api_proto::api::StartServiceRequest::new();
            let path = PathBuf::from(path);
            let mut path = path.canonicalize().unwrap();
            if !path.ends_with(".toml") {
                path = path.join("panditfile.toml");
            }
            let cfg = Config::builder()
                .add_source(config::File::with_name(path.to_str().unwrap()))
                .build()
                .unwrap();
            let proto_name = cfg.get_string("service.proto").unwrap();
            let mut proto_path = proto_path.join(proto_name.clone());
            proto_path.set_extension("proto");

            let mut panditfile = File::open(&proto_path).unwrap();
            let mut proto = Vec::<u8>::new();
            panditfile.read_to_end(&mut proto).unwrap();
            req.set_proto(proto);
            req.set_addr(cfg.get_string("service.address").unwrap());
            let name = cfg.get_string("service.name").unwrap();
            req.set_name(name.clone());

            let _resp = client.start_service(&req).unwrap();
            println!(
                "Successfully created service {} with proto {}",
                name, proto_name
            );
        }
    }
}
