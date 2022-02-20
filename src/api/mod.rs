pub mod proto;

use grpcio::RpcStatus;
use grpcio::RpcStatusCode;
use proto::gen::api;
use proto::gen::api_grpc;

use crate::services::Service;

struct ApiServer {}

impl api_grpc::Api for ApiServer {
    fn start_service(
        &mut self,
        ctx: grpcio::RpcContext,
        req: proto::gen::api::StartServiceRequest,
        sink: grpcio::UnarySink<proto::gen::api::StartServiceReply>,
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
        Service::from_file(path, include, writer, broker)
    }
}
