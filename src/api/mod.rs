use std::env::current_dir;
use std::fs::create_dir;
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

use api_proto::api;
use api_proto::api_grpc;
use async_trait::async_trait;
use bollard::network::ConnectNetworkOptions;
use bollard::network::CreateNetworkOptions;
use bollard::Docker;
use crossbeam_channel;
use dashmap::DashMap;
use grpcio::RpcStatus;
use grpcio::RpcStatusCode;
use std::io::prelude::*;
use tempfile::tempdir;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use crate::broker::Broker;
use crate::server::IntraServer;
use crate::services::Service;
use crate::services::ServiceResult;
use crate::writers::writer_from_proto;

pub struct ApiServer {
    broker: Arc<Broker>,
    server: Arc<RwLock<IntraServer>>,
    network: Option<Arc<dyn NetworkRuntime>>,
}

impl api_grpc::Api for ApiServer {
    fn start_service(
        &mut self,
        ctx: grpcio::RpcContext,
        req: api::StartServiceRequest,
        sink: grpcio::UnarySink<api::StartServiceReply>,
    ) {
        let (tx, rx) = crossbeam_channel::bounded(1);
        match self.network.clone() {
            Some(nw) => {
                let container_id = req.container_id.clone();
                ctx.spawn(async move {
                    nw.create_network(container_id, tx).await.unwrap();
                });
            }
            None => {
                tx.send("127.0.0.1".to_string()).unwrap();
            }
        }
        let host = match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(v) => v,
            Err(err) => {
                sink.fail(RpcStatus::with_message(
                    RpcStatusCode::INTERNAL,
                    format!("an error occurred creating the network: {}", err),
                ));
                return;
            }
        };
        let addr = format!("{}:{}", host, req.port);
        match self.handle_start_service(&ctx, &req, &addr) {
            Ok(_) => {
                let save = serde_json::json!({
                    "name": req.name.clone(),
                    "proto": req.proto.clone(),
                    "port": req.port.clone(),
                    "container_id":req.container_id.clone(),
                });
                let save = serde_json::to_vec(&save).unwrap();
                let mut save_file_path = current_dir().unwrap().join(req.name);
                save_file_path.set_extension("pandit_service");
                let mut save_file = File::create(save_file_path).unwrap();
                save_file.write_all(&save[..]).unwrap();

                sink.success(api::StartServiceReply::new());
            }
            Err(err) => {
                sink.fail(RpcStatus::with_message(
                    RpcStatusCode::INTERNAL,
                    format!("an internal error occurred: {}", err),
                ));
                return;
            }
        };
    }
}

impl Clone for ApiServer {
    fn clone(&self) -> Self {
        Self {
            broker: self.broker.clone(),
            server: self.server.clone(),
            network: self.network.clone(),
        }
    }
}

impl ApiServer {
    pub fn new(
        broker: Arc<Broker>,
        server: Arc<RwLock<IntraServer>>,
        network: Option<Arc<dyn NetworkRuntime>>,
    ) -> Self {
        Self {
            broker,
            server,
            network,
        }
    }

    #[inline(always)]
    fn handle_start_service(
        &mut self,
        ctx: &grpcio::RpcContext,
        req: &api::StartServiceRequest,
        addr: &String,
    ) -> ServiceResult<()> {
        let proto_dir = tempdir()?;
        create_dir(proto_dir.path().join("format"))?;

        let proto_path = proto_dir.path().join("api.proto");
        {
            let mut proto_file = File::create(proto_path.clone())?;
            proto_file.write_all(&req.proto[..])?;
        }
        for (name, file) in proto_libraries() {
            let mut proto_path = proto_dir.path().join(name);
            proto_path.set_extension("proto");
            let mut proto_file = File::create(proto_path.clone())?;
            proto_file.write_all(file)?;
        }
        let service = Service::from_file(
            proto_path.to_str().unwrap_or_default(),
            &[proto_dir.path().to_str().unwrap_or_default()],
            writer_from_proto(
                proto_path.clone(),
                &[proto_dir.path().to_path_buf()],
                addr.as_str(),
            )?,
            self.broker.clone(),
        )?;

        let broker = self.broker.clone();
        let server = self.server.clone();
        let name = req.name.clone();
        ctx.spawn(async move {
            {
                broker.sub_service(&name, &service).unwrap();
            }
            {
                let mut server = server.write().await;
                server.add_servivce(name, service);
            }
        });
        Ok(())
    }
}

fn proto_libraries() -> [(&'static str, &'static [u8]); 3] {
    [
        ("pandit", include_bytes!("../proto/pandit.proto")),
        ("handler", include_bytes!("../proto/handler.proto")),
        (
            "format/http.proto",
            include_bytes!("../proto/format/http.proto"),
        ),
    ]
}

#[async_trait]
pub trait NetworkRuntime: Send + Sync {
    async fn create_network(
        &self,
        container_id: String,
        tx: crossbeam_channel::Sender<String>,
    ) -> ServiceResult<()>;
}

pub struct DockerNetworkRuntime {
    client: Docker,
}

impl DockerNetworkRuntime {
    pub fn new(client: Docker) -> Self {
        Self { client }
    }
}

#[async_trait]
impl NetworkRuntime for DockerNetworkRuntime {
    async fn create_network(
        &self,
        container_id: String,
        tx: crossbeam_channel::Sender<String>,
    ) -> ServiceResult<()> {
        let mut cfg = CreateNetworkOptions::<String>::default();
        let network_name = format!("pandit_network_{}", container_id);
        cfg.name = network_name.clone();
        self.client.create_network(cfg).await?;

        let mut cfg = ConnectNetworkOptions::<String>::default();
        cfg.container = container_id.clone();
        self.client
            .connect_network(network_name.as_str(), cfg)
            .await?;

        let id = hostname::get()?.to_str().unwrap_or_default().to_string();
        let mut cfg = ConnectNetworkOptions::<String>::default();
        cfg.container = id;
        self.client
            .connect_network(network_name.as_str(), cfg)
            .await?;

        let network = self
            .client
            .inspect_network::<String>(network_name.as_str(), None)
            .await?;
        let containers = network.containers.as_ref().unwrap();
        let container_network = containers.get(&container_id).unwrap();

        tx.send(container_network.ipv4_address.clone().unwrap())?;
        Ok(())
    }
}
