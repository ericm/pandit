FROM rust

RUN apt update && apt install -y cmake clang protobuf-compiler

RUN rustup toolchain install nightly

RUN rustup default nightly

RUN rustup component add rustfmt --toolchain nightly