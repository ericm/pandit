use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use crate::proto::gen::format;
use crate::proto::gen::format::http::exts::http_service;
use crate::proto::gen::format::postgres::exts::postgres_service;
use crate::services::{ServiceError, ServiceResult, WriterRef};

use self::http::HttpWriter;

pub mod http;
pub mod postgres;

pub fn writer_from_proto(
    proto_path: PathBuf,
    includes: &[PathBuf],
    hosts: HashSet<String>,
    port: i32,
) -> ServiceResult<WriterRef> {
    let parsed = protobuf_parse::pure::parse_and_typecheck(includes, &[proto_path.clone()])?;
    let filename = proto_path.file_name().unwrap().to_str().unwrap();
    let file = parsed
        .file_descriptors
        .iter()
        .find(|&x| {
            let name = x.get_name().to_string();
            name.ends_with(filename)
        })
        .unwrap();
    let service = file.service.first().unwrap();

    let lb = LoadBalancer::new(hosts, port);

    // Generate writer.
    let options = service.options.as_ref().unwrap_or_default();
    match http_service.get(&options) {
        Some(service) => {
            let version = service.version.unwrap();
            return Ok(Box::new(Mutex::new(HttpWriter::new(lb, version))));
        }
        None => {}
    };
    match postgres_service.get(&options) {
        Some(_) => return Ok(Box::new(Mutex::new(postgres::PostgresWriter::new(lb)?))),
        None => {}
    };

    Err(ServiceError::new("no format defined in service"))
}

pub struct LoadBalancer { // TODO: Fix
    ips: Arc<RwLock<VecDeque<String>>>,
    port: i32,
}

impl LoadBalancer {
    pub fn new(hosts: HashSet<String>, port: i32) -> Self {
        let mut ips = VecDeque::with_capacity(hosts.len());
        for host in hosts {
            ips.push_front(host);
        }
        Self {
            port,
            ips: Arc::new(RwLock::new(ips)),
        }
    }
    pub async fn get_addr(&self) -> String {
        let ip = self.balance().await;
        format!("{}:{}", ip, self.port)
    }

    async fn balance(&self) -> String {
        let mut ips = self.ips.write().await;
        let ip = ips.pop_back().unwrap();
        ips.push_front(ip.clone());
        ip
    }
}
