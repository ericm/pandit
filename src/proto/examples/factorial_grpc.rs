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

const METHOD_FACTORIAL_SERVICE_GET_FACTORIAL: ::grpcio::Method<super::factorial::FactorialRequest, super::factorial::FactorialResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/factorial.FactorialService/GetFactorial",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct FactorialServiceClient {
    client: ::grpcio::Client,
}

impl FactorialServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        FactorialServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn get_factorial_opt(&self, req: &super::factorial::FactorialRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::factorial::FactorialResponse> {
        self.client.unary_call(&METHOD_FACTORIAL_SERVICE_GET_FACTORIAL, req, opt)
    }

    pub fn get_factorial(&self, req: &super::factorial::FactorialRequest) -> ::grpcio::Result<super::factorial::FactorialResponse> {
        self.get_factorial_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_factorial_async_opt(&self, req: &super::factorial::FactorialRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::factorial::FactorialResponse>> {
        self.client.unary_call_async(&METHOD_FACTORIAL_SERVICE_GET_FACTORIAL, req, opt)
    }

    pub fn get_factorial_async(&self, req: &super::factorial::FactorialRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::factorial::FactorialResponse>> {
        self.get_factorial_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait FactorialService {
    fn get_factorial(&mut self, ctx: ::grpcio::RpcContext, req: super::factorial::FactorialRequest, sink: ::grpcio::UnarySink<super::factorial::FactorialResponse>);
}

pub fn create_factorial_service<S: FactorialService + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_FACTORIAL_SERVICE_GET_FACTORIAL, move |ctx, req, resp| {
        instance.get_factorial(ctx, req, resp)
    });
    builder.build()
}
