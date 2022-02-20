// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_API_START_SERVICE: ::grpcio::Method<super::api::StartServiceRequest, super::api::StartServiceReply> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/api.API/StartService",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct ApiClient {
    client: ::grpcio::Client,
}

impl ApiClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        ApiClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn start_service_opt(&self, req: &super::api::StartServiceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::api::StartServiceReply> {
        self.client.unary_call(&METHOD_API_START_SERVICE, req, opt)
    }

    pub fn start_service(&self, req: &super::api::StartServiceRequest) -> ::grpcio::Result<super::api::StartServiceReply> {
        self.start_service_opt(req, ::grpcio::CallOption::default())
    }

    pub fn start_service_async_opt(&self, req: &super::api::StartServiceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::StartServiceReply>> {
        self.client.unary_call_async(&METHOD_API_START_SERVICE, req, opt)
    }

    pub fn start_service_async(&self, req: &super::api::StartServiceRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::StartServiceReply>> {
        self.start_service_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Api {
    fn start_service(&mut self, ctx: ::grpcio::RpcContext, _req: super::api::StartServiceRequest, sink: ::grpcio::UnarySink<super::api::StartServiceReply>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_api<S: Api + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_API_START_SERVICE, move |ctx, req, resp| {
        instance.start_service(ctx, req, resp)
    });
    builder.build()
}
