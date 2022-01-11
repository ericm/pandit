use std::sync::Arc;

use crate::services;
use grpcio;

pub struct Server {}

pub async fn run(service: services::Service) -> Result<(), Box<dyn std::error::Error>> {
    let server_builder = grpcio::ServerBuilder::new(Arc::new(grpcio::Environment::new(2)));
    let service_builder = grpcio::ServiceBuilder::new();
    let glob = grpcio::Method {
        ty: grpcio::MethodType::Unary,
        name: "all",
    };
    for (name, method) in service.methods {
        service_builder.add_unary_handler(
            grpcio::Method {
                ty: grpcio::MethodType::Unary,
                name,
            },
            handler,
        )
    }
    let mut server = server_builder
        .register_service(service_builder.build())
        .build()
        .unwrap();
    server.start();
    for (addr, port) in server.bind_addrs() {
        println!("Connection from {}:{}", addr, port);
    }
    Ok(())
}
