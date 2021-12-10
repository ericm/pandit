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

use dashmap::DashMap;
use futures::stream::StreamExt;
use httparse::Response;
use httparse::EMPTY_HEADER;
use probes::entrypoint::Conn;
use redbpf::load::Loader;
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
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
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

type ConnMap = Arc<DashMap<Conn, (Option<usize>, Vec<u8>)>>;

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
        let resp_pool: ConnMap = Arc::new(DashMap::new());

        tokio::spawn(async move {
            loop {
                let mut buf = vec![0; 1500];
                let (conn, n) = tokio::join!(rx.recv(), stream.read(&mut buf));
                let n = n.unwrap();
                if n == 0 {
                    return;
                }
                let resp_pool = resp_pool.clone();
                tokio::spawn(async move {
                    println!("runn");
                    process_packet(resp_pool, &buf[0..n], &conn.unwrap()).await;
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
        break;
    }
    let ctrlc_fut = async {
        ctrl_c().await.unwrap();
    };
    println!("Hit Ctrl-C to quit");
    ctrlc_fut.await;
}

async fn process_packet<'a>(resp_pool: ConnMap, buf: &'a [u8], conn: &'a Conn) {
    let mut headers = [EMPTY_HEADER; 64];
    let mut resp = Response::new(&mut headers);
    let loc: usize = conn.pld_loc.try_into().unwrap();
    let mut pld = buf[loc..].to_vec();
    let mut body_loc: Option<usize> = None;
    match resp_pool.get_mut(conn) {
        Some(mut conn) => {
            let (prev_body_loc, part_pld) = conn.value_mut();
            println!("prev");
            part_pld.append(&mut pld);
            pld = part_pld.clone();
            body_loc = *prev_body_loc;
        }
        _ => (),
    }
    let pld_str = pld.clone();
    println!("httpld: {}", String::from_utf8(pld_str).unwrap());
    if body_loc.is_none() {
        println!("none");
        let status = resp.parse(pld.as_slice()).unwrap();
        if status.is_complete() {
            println!("complete");
            body_loc = Some(status.unwrap());
        }
    }
    if let (Some(loc), Some(body_size)) =
        (body_loc, get_header::<usize>(&headers, "content-length"))
    {
        println!("loc {} {} {}", loc, body_size, pld.len());
        if pld.len() - loc == body_size {
            println!("complete pld");
        } else {
            println!("incomplete pld");
            resp_pool.insert(*conn, (body_loc, pld));
        }
    } else {
        resp_pool.insert(*conn, (body_loc, pld));
    }
    println!("done");
}

fn get_header<T: str::FromStr>(headers: &[httparse::Header; 64], key: &str) -> Option<T>
where
    <T as str::FromStr>::Err: std::fmt::Debug,
{
    for hdr in headers {
        if hdr == &EMPTY_HEADER {
            break;
        }
        if hdr.name.to_lowercase() == key {
            let val = str::from_utf8(hdr.value).unwrap();
            let val: T = val.parse().unwrap();
            return Some(val);
        }
    }
    None
}

fn probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/bpf/programs/entrypoint/entrypoint.elf"
    ))
}
