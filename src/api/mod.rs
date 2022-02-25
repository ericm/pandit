use std::env::current_dir;
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
use tokio::sync::RwLock;

use crate::broker::Broker;
use crate::server::IntraServer;
use crate::services::Service;
use crate::services::ServiceResult;
use crate::writers::writer_from_proto;

pub struct ApiServer {
    broker: Arc<RwLock<Broker>>,
    server: Arc<RwLock<IntraServer>>,
}

impl api_grpc::Api for ApiServer {
    fn start_service(
        &mut self,
        ctx: grpcio::RpcContext,
        req: api::StartServiceRequest,
        sink: grpcio::UnarySink<api::StartServiceReply>,
    ) {
        match self.handle_start_service(&ctx, &req) {
            Ok(_) => {
                let save = serde_json::json!({
                    "name": req.name.clone(),
                    "proto": req.proto.clone(),
                    "addr": req.addr.clone(),
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
        }
    }
}

impl ApiServer {
    pub fn new(broker: Arc<RwLock<Broker>>, server: Arc<RwLock<IntraServer>>) -> Self {
        Self { broker, server }
    }

    #[inline(always)]
    fn handle_start_service(
        &mut self,
        ctx: &grpcio::RpcContext,
        req: &api::StartServiceRequest,
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
                req.addr.as_str(),
            )?,
            self.broker.clone(),
        )?;

        let broker = self.broker.clone();
        let server = self.server.clone();
        let name = req.name.clone();
        ctx.spawn(async move {
            {
                let mut broker = broker.write().await;
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
