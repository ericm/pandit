#![feature(destructuring_assignment)]
#![feature(ptr_to_from_bits)]
#![feature(binary_heap_into_iter_sorted)]

pub mod broker;
pub mod handlers;
pub mod proto;
pub mod server;
pub mod services;
pub mod writers;

use clap;
use dashmap::DashMap;
use futures::stream::StreamExt;
use httparse::Response;
use httparse::EMPTY_HEADER;
use serde_json;
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::fs::OpenOptions;
use std::net::Ipv4Addr;
use std::net::TcpStream as StdStream;
use std::os::unix::prelude::FromRawFd;
use std::process;
use std::ptr;
use std::str;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::signal::ctrl_c;
use tokio::sync::mpsc;
use tokio_test;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    if unsafe { libc::geteuid() != 0 } {
        error!("You must be root to use eBPF!");
        process::exit(1);
    }

    let app = new_app();

    let cfg = services::new_config(app.get_matches().value_of("config").unwrap());

    let iface = "lo";
    println!("Attaching socket to interface {}", iface);

    let ctrlc_fut = async {
        ctrl_c().await.unwrap();
    };
    println!("Hit Ctrl-C to quit");
    ctrlc_fut.await;
}

fn new_app() -> clap::App<'static, 'static> {
    clap::App::new("pandit")
        .version("1.0")
        .author("Eric Moynihan")
        .about("Pandit CLI")
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .default_value("./.pandit.yml"),
        )
}
