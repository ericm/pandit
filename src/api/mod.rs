use std::fs::create_dir;
use std::fs::File;
use std::sync::Arc;

use api_proto::api;
use api_proto::api_grpc;
use dashmap::DashMap;
use grpcio::RpcStatus;
use grpcio::RpcStatusCode;
use std::io::prelude::*;
use tempfile::tempdir;
use tokio::sync::Mutex;

use crate::broker::Broker;
use crate::server::IntraServer;
use crate::services::Service;
use crate::services::ServiceResult;
use crate::writers::writer_from_proto;

pub struct ApiServer {
    broker: Arc<Mutex<Broker>>,
    server: Arc<Mutex<IntraServer>>,
}

impl api_grpc::Api for ApiServer {
    fn start_service(
        &mut self,
        _ctx: grpcio::RpcContext,
        req: api::StartServiceRequest,
        sink: grpcio::UnarySink<api::StartServiceReply>,
    ) {
        // let cfg = match config::Config::try_from(&req.config) {
        //     Ok(c) => c,
        //     Err(err) => {
        //         sink.fail(RpcStatus::with_message(
        //             RpcStatusCode::INVALID_ARGUMENT,
        //             format!("Config file unable to be parsed: {}", err),
        //         ));
        //         return;
        //     }
        // };
        match self.handle_start_service(&req) {
            Ok(_) => {
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
        }
    }
}

impl ApiServer {
    pub fn new(broker: Arc<Mutex<Broker>>, server: Arc<Mutex<IntraServer>>) -> Self {
        Self { broker, server }
    }

    #[inline(always)]
    fn handle_start_service(&mut self, req: &api::StartServiceRequest) -> ServiceResult<()> {
        let proto_dir = tempdir()?;
        create_dir(proto_dir.path().join("format"))?;

        let proto_path = proto_dir.path().join("api.proto");
        {
            let mut proto_file = File::create(proto_path.clone())?;
            proto_file.write_all(&req.proto[..])?;
        }
        let service = Service::from_file(
            proto_path.to_str().unwrap_or_default(),
            &[proto_dir.path().to_str().unwrap_or_default()],
            writer_from_proto(
                proto_path.clone(),
                &[proto_dir.path().to_path_buf()],
                req.addr.as_str(),
            )?,
            self.broker.clone(),
        )?;

        for (name, file) in proto_libraries() {
            let proto_path = proto_dir.path().join(*name);
            let mut proto_file = File::create(proto_path.clone())?;
            proto_file.write_all(*file)?;
        }

        let broker = self.broker.clone();
        let server = self.server.clone();
        tokio::spawn(async move {
            let mut broker = broker.lock().await;
            broker.sub_service(&service).unwrap();
            let mut server = server.lock().await;
            server.add_servivce(service);
        });
        Ok(())
    }
}

fn proto_libraries() -> &'static [(&'static str, &'static [u8])] {
    &[
        ("pandit", include_bytes!("../proto/pandit.proto")),
        ("handler", include_bytes!("../proto/handler.proto")),
        (
            "format/http.proto",
            include_bytes!("../proto/format/http.proto"),
        ),
    ]
}
