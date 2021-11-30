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
use std::convert::TryInto;
use std::env;
use std::net::TcpStream as StdStream;
use std::os::unix::prelude::FromRawFd;
use std::process;
use std::ptr;
use std::str;
use tokio;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::signal::ctrl_c;
use tokio::sync::mpsc;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

use httparse::Response;
use httparse::EMPTY_HEADER;
use probes::entrypoint::Conn;
use redbpf::load::Loader;
use std::collections::HashMap;

type ConnMap = HashMap<Conn, Vec<u8>>;

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
        if sf.name() != "measure_tcp_lifetime" {
            continue;
        }
        if let Ok(sock_raw_fd) = sf.attach_socket_filter(iface) {
            println!("sock fd: {}", sock_raw_fd);
            let stream = unsafe { StdStream::from_raw_fd(sock_raw_fd) };
            streams.push(TcpStream::from_std(stream).unwrap());
        }
    }
    for mut stream in streams {
        let (tx, mut rx) = mpsc::channel(32);
        let mut resp_pool: ConnMap = HashMap::new();
        let (hash_tx, mut hash_rx) = mpsc::channel(32);

        tokio::spawn(async move {
            loop {
                let hash_tx = hash_tx.clone();
                let mut buf = vec![0; 1500];
                let (conn, n) = tokio::join!(rx.recv(), stream.read(&mut buf));
                let n = n.unwrap();
                if n == 0 {
                    return;
                }
                tokio::spawn(async move {
                    process_packet(&hash_tx, &buf[0..n], &conn.unwrap()).await;
                });
            }
        });
        tokio::spawn(async move {
            while let Some((name, events)) = loaded.events.next().await {
                match name.as_str() {
                    "established" => {
                        println!("found established");
                        for event in events {
                            let conn = unsafe { ptr::read(event.as_ptr() as *const Conn) };
                            println!("conn {}", conn.pld_loc);
                            tx.send(conn).await.expect("problem sending");
                        }
                    }
                    _ => {
                        error!("unknown event = {}", name);
                    }
                }
                println!("{}", name);
            }
        });
        while let Some((conn, buf)) = hash_rx.recv().await {
            resp_pool.insert(conn, buf);
        }
        break;
    }
    let ctrlc_fut = async {
        ctrl_c().await.unwrap();
    };
    println!("Hit Ctrl-C to quit");
    ctrlc_fut.await;
}

async fn process_packet<'a>(tx: &'a mpsc::Sender<(Conn, Vec<u8>)>, buf: &'a [u8], conn: &'a Conn) {
    let mut headers = [EMPTY_HEADER; 64];
    let mut resp = Response::new(&mut headers);
    let loc: usize = conn.pld_loc.try_into().unwrap();
    let pld = buf[loc..].to_vec();
    if resp.parse(pld.as_slice()).unwrap().is_partial() {
        println!("partial");
        tx.send((*conn, pld)).await.expect("problem sending");
    } else {
        println!("version: {}", resp.version.unwrap());
    }
}

fn probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/bpf/programs/entrypoint/entrypoint.elf"
    ))
}
