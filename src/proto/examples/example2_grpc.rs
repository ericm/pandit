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

const METHOD_POSTGRE_SQL_GET_EXAMPLE: ::grpcio::Method<super::example2::ExampleTable, super::example2::Empty> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/pg_demo.PostgreSQL/GetExample",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct PostgreSqlClient {
    client: ::grpcio::Client,
}

impl PostgreSqlClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        PostgreSqlClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn get_example_opt(&self, req: &super::example2::ExampleTable, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::example2::Empty> {
        self.client.unary_call(&METHOD_POSTGRE_SQL_GET_EXAMPLE, req, opt)
    }

    pub fn get_example(&self, req: &super::example2::ExampleTable) -> ::grpcio::Result<super::example2::Empty> {
        self.get_example_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_example_async_opt(&self, req: &super::example2::ExampleTable, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::example2::Empty>> {
        self.client.unary_call_async(&METHOD_POSTGRE_SQL_GET_EXAMPLE, req, opt)
    }

    pub fn get_example_async(&self, req: &super::example2::ExampleTable) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::example2::Empty>> {
        self.get_example_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait PostgreSql {
    fn get_example(&mut self, ctx: ::grpcio::RpcContext, req: super::example2::ExampleTable, sink: ::grpcio::UnarySink<super::example2::Empty>);
}

pub fn create_postgre_sql<S: PostgreSql + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_POSTGRE_SQL_GET_EXAMPLE, move |ctx, req, resp| {
        instance.get_example(ctx, req, resp)
    });
    builder.build()
}
