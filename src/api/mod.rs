use std::env::current_dir;
use std::error::Error;
use std::fs::create_dir;
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

use api_proto::api;
use api_proto::api_grpc;
use async_trait::async_trait;
use bollard::models::Network;
use bollard::network::ConnectNetworkOptions;
use bollard::network::CreateNetworkOptions;
use bollard::network::DisconnectNetworkOptions;
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
use crate::services::ServiceError;
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
        let host = match self.network.clone() {
            Some(nw) => {
                let container_id = req.container_id.clone();
                let host = match nw.create_network(container_id) {
                    Ok(host) => host,
                    Err(err) => {
                        sink.fail(RpcStatus::with_message(
                            RpcStatusCode::INTERNAL,
                            format!("an error occurred creating the network: {}", err),
                        ));
                        return;
                    }
                };
                host
            }
            None => "127.0.0.1".to_string(),
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
    fn create_network(&self, container_id: String) -> ServiceResult<String>;
    async fn run(&self);
}

pub struct DockerNetworkRuntime {
    client: Docker,
    rx: crossbeam_channel::Receiver<String>,
    tx: crossbeam_channel::Sender<String>,
    hostrx: crossbeam_channel::Receiver<String>,
    hosttx: crossbeam_channel::Sender<String>,
    erx: crossbeam_channel::Receiver<String>,
    etx: crossbeam_channel::Sender<String>,
}

impl DockerNetworkRuntime {
    pub fn new(client: Docker) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let (hosttx, hostrx) = crossbeam_channel::unbounded();
        let (etx, erx) = crossbeam_channel::unbounded();
        Self {
            client,
            tx,
            rx,
            hosttx,
            hostrx,
            etx,
            erx,
        }
    }
}

#[async_trait]
impl NetworkRuntime for DockerNetworkRuntime {
    async fn run(&self) {
        loop {
            let container_id = match self.rx.recv() {
                Ok(v) => v,
                Err(err) => {
                    eprintln!(
                        "an error occurred receiving container id in docker network runtime: {}",
                        err
                    );
                    continue;
                }
            };
            match self._create_network(container_id).await {
                Ok(host) => {
                    self.hosttx.send(host).unwrap();
                }
                Err(err) => {
                    self.etx.send(err.to_string()).unwrap();
                }
            }
        }
    }

    fn create_network(&self, container_id: String) -> ServiceResult<String> {
        self.tx.send(container_id)?;
        crossbeam_channel::select! {
            recv(self.hostrx) -> host => {
                return Ok(host?);
            },
            recv(self.erx) -> err => {
                return Err(ServiceError::new(err?.as_str()));
            }
        }
    }
}

impl DockerNetworkRuntime {
    async fn _create_network(&self, container_id: String) -> ServiceResult<String> {
        let network_name = format!("pandit_network_{}", container_id);

        {
            let networks = self.client.list_networks::<String>(None).await?;
            let network = networks
                .iter()
                .find(|v| v.name == Some(network_name.clone()));
            if let Some(network) = network {
                return self
                    .get_ipv4(
                        network.name.as_ref().unwrap_or(&"".to_string()),
                        &container_id,
                    )
                    .await;
            }
        };

        let mut cfg = CreateNetworkOptions::<String>::default();
        cfg.name = network_name.clone();
        self.client.create_network(cfg).await?;

        self.connect_containers(&network_name, &container_id)
            .await?;

        self.get_ipv4(&network_name, &container_id).await
    }

    async fn connect_containers(
        &self,
        network_name: &String,
        container_id: &String,
    ) -> ServiceResult<()> {
        let id = hostname::get()?.to_str().unwrap_or_default().to_string();
        let container_info = self.client.inspect_container(id.as_str(), None).await?;
        let pandit_name = container_info.name.ok_or("no name for pandit container")?;

        // Disconnect networks just in case.
        let mut cfg = DisconnectNetworkOptions::<String>::default();
        cfg.container = pandit_name.clone();
        match self
            .client
            .disconnect_network(network_name.as_str(), cfg)
            .await
        {
            Ok(_) => {
                println!("disconnected from network: {}", pandit_name);
            }
            Err(_) => {}
        }

        let mut cfg = DisconnectNetworkOptions::<String>::default();
        cfg.container = container_id.clone();
        match self
            .client
            .disconnect_network(network_name.as_str(), cfg)
            .await
        {
            Ok(_) => {
                println!("disconnected from network: {}", container_id);
            }
            Err(_) => {}
        }

        // Connect networks.
        let mut cfg = ConnectNetworkOptions::<String>::default();
        cfg.container = container_id.clone();
        self.client
            .connect_network(network_name.as_str(), cfg)
            .await?;
        let mut cfg = ConnectNetworkOptions::<String>::default();
        cfg.container = id;
        self.client
            .connect_network(network_name.as_str(), cfg)
            .await?;
        Ok(())
    }

    async fn get_ipv4(
        &self,
        network_name: &String,
        container_id: &String,
    ) -> ServiceResult<String> {
        for _ in 0..2 {
            let network = self
                .client
                .inspect_network::<String>(network_name.as_str(), None)
                .await?;
            let container_info = self
                .client
                .inspect_container(container_id.as_str(), None)
                .await?;
            let container_id = container_info.id.ok_or("container has no id")?;
            let container_name = container_info.name.ok_or("container has no id")?;
            let containers = network
                .containers
                .as_ref()
                .ok_or(ServiceError::new("no containers for network"))?;

            let container_network = match containers.get(&container_id) {
                Some(v) => v,
                None => {
                    self.connect_containers(network.name.as_ref().unwrap(), &container_name)
                        .await?;
                    continue;
                }
            };

            return Ok(container_network
                .ipv4_address
                .clone()
                .ok_or(ServiceError::new("no ipv4 address for network"))?);
        }
        Err(ServiceError::new(
            "get_ipv4 counldnt find the container in the network",
        ))
    }
}
