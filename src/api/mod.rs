use std::fs::File;
use std::sync::Arc;

use api_proto::api;
use api_proto::api_grpc;
use grpcio::RpcStatus;
use grpcio::RpcStatusCode;
use std::io::prelude::*;
use tempfile::tempdir;
use tokio::sync::Mutex;

use crate::broker::Broker;
use crate::services::Service;
use crate::services::ServiceResult;
use crate::writers::writer_from_proto;

struct ApiServer {
    broker: Arc<Mutex<Broker>>,
}

impl api_grpc::Api for ApiServer {
    fn start_service(
        &mut self,
        ctx: grpcio::RpcContext,
        req: api::StartServiceRequest,
        sink: grpcio::UnarySink<api::StartServiceReply>,
    ) {
        let cfg = match config::Config::try_from(&req.config) {
            Ok(c) => c,
            Err(err) => {
                sink.fail(RpcStatus::with_message(
                    RpcStatusCode::INVALID_ARGUMENT,
                    format!("Config file unable to be parsed: {}", err),
                ));
                return;
            }
        };
        match self.handle_start_service(&req) {
            Ok(_) => {}
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

impl ApiServer {
    fn handle_start_service(&mut self, req: &api::StartServiceRequest) -> ServiceResult<()> {
        let proto_dir = tempdir()?;
        let proto_path = proto_dir.path().join("api.proto");
        {
            let mut proto_file = File::create(proto_path.clone())?;
            proto_file.write_all(&req.proto[..])?;
        }

        let service = Service::from_file(
            proto_path.to_str().unwrap_or_default(),
            &[proto_dir.path().to_str().unwrap_or_default()],
            writer_from_proto(&proto_path)?,
            self.broker.clone(),
        )?;
        let broker = self.broker.clone();
        tokio::spawn(async move {
            let mut broker = broker.lock().await;
            broker.sub_service(&service).unwrap();
        });
        Ok(())
    }
}
