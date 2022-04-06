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

const METHOD_POSTGRE_NUM_STORE_SET_NUMBER: ::grpcio::Method<super::postgres_numstore::NumberTable, super::postgres_numstore::Empty> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/pg_num.PostgreNumStore/SetNumber",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_POSTGRE_NUM_STORE_GET_NUMBER: ::grpcio::Method<super::postgres_numstore::NumberTable, super::postgres_numstore::NumberTable> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/pg_num.PostgreNumStore/GetNumber",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct PostgreNumStoreClient {
    client: ::grpcio::Client,
}

impl PostgreNumStoreClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        PostgreNumStoreClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn set_number_opt(&self, req: &super::postgres_numstore::NumberTable, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::postgres_numstore::Empty> {
        self.client.unary_call(&METHOD_POSTGRE_NUM_STORE_SET_NUMBER, req, opt)
    }

    pub fn set_number(&self, req: &super::postgres_numstore::NumberTable) -> ::grpcio::Result<super::postgres_numstore::Empty> {
        self.set_number_opt(req, ::grpcio::CallOption::default())
    }

    pub fn set_number_async_opt(&self, req: &super::postgres_numstore::NumberTable, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::postgres_numstore::Empty>> {
        self.client.unary_call_async(&METHOD_POSTGRE_NUM_STORE_SET_NUMBER, req, opt)
    }

    pub fn set_number_async(&self, req: &super::postgres_numstore::NumberTable) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::postgres_numstore::Empty>> {
        self.set_number_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_number_opt(&self, req: &super::postgres_numstore::NumberTable, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::postgres_numstore::NumberTable> {
        self.client.unary_call(&METHOD_POSTGRE_NUM_STORE_GET_NUMBER, req, opt)
    }

    pub fn get_number(&self, req: &super::postgres_numstore::NumberTable) -> ::grpcio::Result<super::postgres_numstore::NumberTable> {
        self.get_number_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_number_async_opt(&self, req: &super::postgres_numstore::NumberTable, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::postgres_numstore::NumberTable>> {
        self.client.unary_call_async(&METHOD_POSTGRE_NUM_STORE_GET_NUMBER, req, opt)
    }

    pub fn get_number_async(&self, req: &super::postgres_numstore::NumberTable) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::postgres_numstore::NumberTable>> {
        self.get_number_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait PostgreNumStore {
    fn set_number(&mut self, ctx: ::grpcio::RpcContext, req: super::postgres_numstore::NumberTable, sink: ::grpcio::UnarySink<super::postgres_numstore::Empty>);
    fn get_number(&mut self, ctx: ::grpcio::RpcContext, req: super::postgres_numstore::NumberTable, sink: ::grpcio::UnarySink<super::postgres_numstore::NumberTable>);
}

pub fn create_postgre_num_store<S: PostgreNumStore + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_POSTGRE_NUM_STORE_SET_NUMBER, move |ctx, req, resp| {
        instance.set_number(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_POSTGRE_NUM_STORE_GET_NUMBER, move |ctx, req, resp| {
        instance.get_number(ctx, req, resp)
    });
    builder.build()
}
