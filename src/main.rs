// This program can be executed by
// # cargo run --example tcp-lifetime [interface]
// It reports (saddr, sport, daddr, dport, lifetime) of which established and
// closed while the program is running.

// Example of execution
// $ sudo -E cargo run --example tcp-lifetime wlp0s20f3
// Attaching socket to interface wlp0s20f3
// Hit Ctrl-C to quit
//          src           →           dst          |  duration
// 192.168. 0 . 9 :36940  →   8 . 8 . 8 . 8 :53    |     1303 ms
//  8 . 8 . 8 . 8 :53     →  192.168. 0 . 9 :36940 |     1304 ms

use futures::stream::StreamExt;
use std::env;
use std::net::TcpStream as StdStream;
use std::os::unix::prelude::FromRawFd;
use std::process;
use std::str;
use tokio;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::signal::ctrl_c;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

use probes::entrypoint::SocketAddr;
use redbpf::load::Loader;

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

    let args: Vec<String> = env::args().collect();
    let iface = match args.get(1) {
        Some(val) => val,
        None => "lo",
    };
    println!("Attaching socket to interface {}", iface);
    let mut streams = Vec::new();
    let mut loaded = Loader::load(probe_code()).expect("error loading BPF program");
    for sf in loaded.socket_filters_mut() {
        if let Ok(sock_raw_fd) = sf.attach_socket_filter(iface) {
            let stream = unsafe { StdStream::from_raw_fd(sock_raw_fd) };
            streams.push(TcpStream::from_std(stream).unwrap());
        }
    }
    for mut stream in streams {
        tokio::spawn(async move {
            loop {
                let mut buf = vec![0; 1500];
                let n = stream.read(&mut buf).await.unwrap();
                if n == 0 {
                    return;
                }
                tokio::spawn(async move {
                    process_packet(&buf[0..n]);
                });
            }
        });
    }
    let ctrlc_fut = async {
        ctrl_c().await.unwrap();
    };
    println!("Hit Ctrl-C to quit");
    ctrlc_fut.await;
    while let Some((name, events)) = loaded.events.next().await {
        println!("{}", name);
    }
}

fn process_packet(buf: &[u8]) {
    println!("{}", str::from_utf8(buf).unwrap());
}

fn probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/bpf/programs/entrypoint/entrypoint.elf"
    ))
}
