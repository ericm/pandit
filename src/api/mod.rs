use std::convert::{TryFrom, TryInto};
use std::env::current_dir;
use std::error::Error;
use std::fs::create_dir;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use api_proto::api;
use api_proto::api::StartServiceRequest_oneof_container;
use api_proto::api_grpc;
use api_proto::api_grpc::ApiClient;
use async_trait::async_trait;
use bollard::models::Network;
use bollard::network::ConnectNetworkOptions;
use bollard::network::CreateNetworkOptions;
use bollard::network::DisconnectNetworkOptions;
use bollard::Docker;
use crossbeam_channel;
use dashmap::DashMap;
use dashmap::DashSet;
use grpcio::ChannelBuilder;
use grpcio::EnvBuilder;
use grpcio::RpcStatus;
use grpcio::RpcStatusCode;
use k8s_openapi::api::core::v1::{Node, Pod, Service as K8sService};
use kube::runtime::utils::try_flatten_applied;
use kube::runtime::watcher;
use std::io::prelude::*;
use tempfile::tempdir;
use tokio::sync::mpsc;
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
    server: Arc<IntraServer>,
    network: Option<Arc<dyn NetworkRuntime>>,
    k8s_handler: Option<Arc<K8sHandler>>,
}

impl api_grpc::Api for ApiServer {
    fn start_service(
        &mut self,
        ctx: grpcio::RpcContext,
        req: api::StartServiceRequest,
        sink: grpcio::UnarySink<api::StartServiceReply>,
    ) {
        log::info!("received start service request for: {}", req.get_name());
        let default_host = "127.0.0.1".to_string();
        let default = StartServiceRequest_oneof_container::k8s_pod("".to_string());
        let host = match self.network.clone() {
            Some(nw) => {
                let container_id = match req.container.as_ref().unwrap_or(&default) {
                    StartServiceRequest_oneof_container::docker_id(v) => v,
                    _ => {
                        sink.fail(RpcStatus::with_message(
                            RpcStatusCode::INVALID_ARGUMENT,
                            format!("docker mode requires docker container id"),
                        ));
                        return;
                    }
                };
                let host = match nw.create_network(container_id.clone()) {
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
            None => match self.k8s_handler.clone() {
                Some(handler) => {
                    let (tx, rx) = crossbeam_channel::unbounded();
                    let (etx, erx) = crossbeam_channel::unbounded();
                    match handler.call_handle_if_external(&req) {
                        Ok(v) => {
                            tx.send(v).unwrap();
                        }
                        Err(err) => {
                            log::error!("k8s error: {}", err);
                            etx.send(err.to_string()).unwrap();
                        }
                    };
                    crossbeam_channel::select! {
                        recv(rx) -> host => {
                            let host = host.unwrap();
                            match host {
                                Some(host) => host,
                                None => default_host,
                            }
                        },
                        recv(erx) -> err => {
                            log::error!("k8s error returned");
                            sink.fail(RpcStatus::with_message(
                                RpcStatusCode::INTERNAL,
                                format!("an error occurred interfacing with k8s: {}", err.unwrap()),
                            ));
                            return;
                        },
                    }
                }
                None => default_host,
            },
        };
        let addr = format!("{}:{}", host, req.get_port());
        match self.handle_start_service(&ctx, req.clone(), &addr) {
            Ok(_) => {
                let mut save = serde_json::json!({
                    "name": req.get_name(),
                    "proto": req.get_proto(),
                    "port": req.get_port(),
                    "docker_id": "",
                    "k8s_pod": "",
                    "k8s_service": "",
                });
                if req.has_docker_id() {
                    *save.get_mut("docker_id").unwrap() = serde_json::json!(req.get_docker_id());
                } else if req.has_k8s_pod() {
                    *save.get_mut("k8s_pod").unwrap() = serde_json::json!(req.get_k8s_pod());
                } else if req.has_k8s_service() {
                    *save.get_mut("k8s_service").unwrap() =
                        serde_json::json!(req.get_k8s_service());
                }
                let save = serde_json::to_vec(&save).unwrap();
                let mut save_file_path = current_dir().unwrap().join(req.get_name());
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
            k8s_handler: self.k8s_handler.clone(),
        }
    }
}

impl ApiServer {
    pub fn new(
        broker: Arc<Broker>,
        server: Arc<IntraServer>,
        network: Option<Arc<dyn NetworkRuntime>>,
        k8s_handler: Option<Arc<K8sHandler>>,
    ) -> Self {
        Self {
            broker,
            server,
            network,
            k8s_handler,
        }
    }

    #[inline(always)]
    fn handle_start_service(
        &mut self,
        ctx: &grpcio::RpcContext,
        req: api::StartServiceRequest,
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
                broker.publish_service(&name, &service).unwrap();
                if req.has_k8s_pod() {
                    let pod = req.get_k8s_pod();
                    log::info!("Adding pod to watch: {}", pod);
                    broker.add_pod_to_watch(&pod.to_string(), &name);
                }
            }
            log::info!("broker service published: {}", &name);
            {
                server.add_servivce(name, service).await;
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

pub async fn add_service_from_file(
    path: PathBuf,
    k8s_handler: &Option<Arc<K8sHandler>>,
    broker: Option<Arc<Broker>>,
    client: &ApiClient,
) -> ServiceResult<()> {
    let ext = match path.extension() {
        Some(ext) => ext.to_str().unwrap(),
        None => return Ok(()),
    };
    if ext != "pandit_service" {
        return Ok(());
    }
    let save_file = File::open(path).unwrap();
    let save: serde_json::Value = serde_json::from_reader(save_file).unwrap();
    let docker_id = save.get("docker_id").unwrap().as_str().unwrap().to_string();
    let k8s_pod = save.get("k8s_pod").unwrap().as_str().unwrap().to_string();
    let k8s_service = save
        .get("k8s_service")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let name = save.get("name").unwrap().as_str().unwrap().to_string();
    if k8s_pod != "" {
        match broker {
            Some(broker) => {
                broker.add_pod_to_watch(&k8s_pod, &name);
            }
            None => {}
        }
        let on_current = match &k8s_handler {
            Some(handler) => handler.is_pod_on_current(&k8s_pod).await.unwrap(),
            None => true,
        };
        if !on_current {
            return Ok(());
        }
    }
    log::info!("starting service: {}", name);
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
    let mut req = api_proto::api::StartServiceRequest::new();
    req.set_proto(proto);
    req.set_port(port);
    req.set_name(name);
    if k8s_pod != "" {
        req.set_k8s_pod(k8s_pod);
    } else if k8s_service != "" {
        req.set_k8s_service(k8s_service);
    } else if docker_id != "" {
        req.set_docker_id(docker_id);
    }
    client.start_service(&req).unwrap();
    Ok(())
}

pub type K8sResult<T> = Result<T, String>;
#[derive(Clone)]
pub struct K8sHandler {
    pandit_port: u16,
    rx: Arc<RwLock<mpsc::Receiver<api::StartServiceRequest>>>,
    tx: Arc<mpsc::Sender<api::StartServiceRequest>>,
    hostrx: Arc<RwLock<mpsc::Receiver<K8sResult<Option<String>>>>>,
    hosttx: Arc<mpsc::Sender<K8sResult<Option<String>>>>,
}

impl K8sHandler {
    pub async fn new(pandit_port: u16) -> ServiceResult<Self> {
        let (tx, rx) = mpsc::channel(10000);
        let (hosttx, hostrx) = mpsc::channel(10000);
        Ok(Self {
            pandit_port,
            tx: Arc::new(tx),
            rx: Arc::new(RwLock::new(rx)),
            hosttx: Arc::new(hosttx),
            hostrx: Arc::new(RwLock::new(hostrx)),
        })
    }

    fn call_handle_if_external(
        &self,
        req: &api::StartServiceRequest,
    ) -> ServiceResult<Option<String>> {
        let tx = self.tx.clone();
        tx.blocking_send(req.clone())?;
        let mut hostrx = self.hostrx.blocking_write();
        log::info!("k8s_grpcio: call handle if external");
        match hostrx
            .blocking_recv()
            .ok_or("error receiving from k8s handler runtime")?
        {
            Ok(host) => Ok(host),
            Err(err) => Err(ServiceError::new(err.as_str())),
        }
    }

    pub async fn run(&self) {
        loop {
            let mut rx = self.rx.write().await;
            log::info!("k8s: listening for calls to handle_if_external");
            let req = match rx.recv().await {
                Some(v) => v,
                None => {
                    log::error!("an error occurred receiving start service request for k8s",);
                    continue;
                }
            };
            log::info!("k8s: runtime received request");
            let res = match self._handle_if_external(&req).await {
                Ok(v) => Ok(v),
                Err(err) => Err(err.to_string()),
            };
            log::info!("k8s: host found: {:?}", res);
            self.hosttx.send(res).await.unwrap();
        }
    }

    async fn _handle_if_external(
        &self,
        req: &api::StartServiceRequest,
    ) -> ServiceResult<Option<String>> {
        use api_proto::api::StartServiceRequest_oneof_container::*;
        log::info!("k8s: handle if external triggered");
        let client = kube::Client::try_default().await?;
        let current_node = std::env::var("NODE_NAME")?;
        log::info!("k8s: current_node: {}", current_node);
        let ip;
        let pod_node = match req.container.as_ref().ok_or("no container in req")? {
            k8s_pod(id) => {
                let pods: kube::Api<Pod> = kube::Api::default_namespaced(client);
                log::info!("k8s: connected to default namespace pod api");
                let pod: Pod = pods.get_opt(id.as_str()).await?.ok_or("no pod found")?;
                log::info!("k8s: found pod with id: {}", id);
                let spec = pod.spec.ok_or("no pod spec")?;
                ip = pod
                    .status
                    .ok_or("no pod status")?
                    .pod_ip
                    .ok_or("no pod ip")?;
                log::info!("k8s: found pod's ip: {}", &ip);
                spec.node_name.ok_or("no node name")?
            }
            k8s_service(id) => {
                let services: kube::Api<K8sService> = kube::Api::default_namespaced(client);
                let service: K8sService = services.get(id.as_str()).await?;
                let spec = service.spec.ok_or("no pod spec")?;
                // TODO: Add service to all hosts.
                return Ok(Some(spec.cluster_ip.ok_or("no cluster ip")?));
            }
            docker_id(_) => {
                return Err(ServiceError::new("cannot use docker_id with k8s"));
            }
        };
        log::info!("k8s: pod's node: {}", pod_node);
        if pod_node == current_node {
            return Ok(Some(ip));
        }
        let node_ip = {
            let client = kube::Client::try_default().await?;
            let nodes: kube::Api<Node> = kube::Api::default_namespaced(client);
            let node = nodes.get(pod_node.as_str()).await?;
            let status = node.status.ok_or("no node status")?;
            let addresses = status.addresses.ok_or("no node addresses")?;
            let addr = addresses.first().ok_or("no available node addresses")?;
            addr.address.clone()
        };
        log::info!("k8s: node ip: {}", node_ip);
        let client = {
            let node_addr = format!("{}:{}", node_ip, self.pandit_port);
            let env = Arc::new(EnvBuilder::new().build());
            let ch = ChannelBuilder::new(env).connect(node_addr.as_str());
            api_proto::api_grpc::ApiClient::new(ch)
        };
        // TODO: Add k8s watcher for deleted/recreated pods/services.
        client.start_service(req)?;
        Ok(None)
    }

    pub async fn is_pod_on_current(&self, pod: &String) -> ServiceResult<bool> {
        let client = kube::Client::try_default().await?;
        let current_node = std::env::var("NODE_NAME")?;
        let pod_node = {
            let pods: kube::Api<Pod> = kube::Api::default_namespaced(client);
            let pod: Pod = pods.get(pod.as_str()).await?;
            let spec = pod.spec.ok_or("no pod spec")?;
            // TODO: remove this as node_name is optional. Maybe not.
            spec.node_name.ok_or("no node name")?
        };
        Ok(pod_node == current_node)
    }
}

// mod k8s_tests {
//     use http::{Request, Response};
//     use hyper::Body;

//     #[tokio::test(flavor = "current_thread")]
//     async fn test_handle_if_external() {
//         use super::*;
//         use tower_test::mock;
//         let target = K8sHandler::new(1234).await?;
//         let (mock_service, mut handle) = mock::pair::<Request<Body>, Response<Body>>();

//     }
// }

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
                    log::error!(
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
                log::info!("disconnected from network: {}", pandit_name);
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
                log::info!("disconnected from network: {}", container_id);
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
