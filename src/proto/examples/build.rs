use std::{env::current_dir, path::PathBuf};

fn main() {
    let proto_root = PathBuf::from(current_dir().unwrap())
        .join("..")
        .canonicalize()
        .unwrap();
    let output_dir = current_dir().unwrap();
    println!("cargo:rerun-if-changed={}", proto_root.to_str().unwrap());
    protoc_grpcio::compile_grpc_protos(
        &["example1.proto"],
        &[proto_root.to_str().unwrap().to_string()],
        &output_dir,
        None,
    )
    .expect("Failed to compile gRPC definitions!");
}
