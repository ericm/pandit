use std::{io, sync::Arc};

use grpcio::{ChannelBuilder, EnvBuilder};

mod factorial;
mod factorial_grpc;

mod postgres_numstore;
mod postgres_numstore_grpc;

fn main() {
    // Call FactorialService.GetFactorial.
    let fact = {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect("localhost:50122");
        let client = factorial_grpc::FactorialServiceClient::new(ch);

        // Read in stdin.
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let n: i32 = input.trim().parse().unwrap();
        let mut req = factorial::FactorialRequest::new();
        req.set_number(n);
        let resp = client.get_factorial(&req).unwrap();
        resp.response
    };

    // Set value in postgres.
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("localhost:50122");
    let client = postgres_numstore_grpc::PostgreNumStoreClient::new(ch);

    let mut req = postgres_numstore::NumberTable::new();
    req.set_num(fact);
    let _ = client.set_number(&req).unwrap();

    let resp = client.get_number(&req).unwrap();
    println!("The factorial that was calculated by the factorial service and stored in the postgres service is: {}", resp.num);
}
