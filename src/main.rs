#![feature(destructuring_assignment)]

pub mod client;
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
use probes::entrypoint::Conn;
use redbpf::load::Loader;
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

type ConnMap = Arc<DashMap<Conn, (Option<usize>, Option<usize>, Vec<u8>)>>;

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

async fn process_packet<'a>(resp_pool: ConnMap, buf: &'a [u8], conn: &'a Conn) {
    let mut headers = [EMPTY_HEADER; 64];
    let mut resp = Response::new(&mut headers);
    let loc: usize = conn.pld_loc.try_into().unwrap();
    let mut pld = buf[loc..].to_vec();
    let mut body_loc: Option<usize> = None;
    let mut content_size: Option<usize> = None;
    match resp_pool.get_mut(conn) {
        Some(mut conn) => {
            let (prev_body_loc, prev_content_size, part_pld) = conn.value_mut();
            println!("prev");
            part_pld.append(&mut pld);
            pld = part_pld.clone();
            body_loc = *prev_body_loc;
            content_size = *prev_content_size;
        }
        _ => (),
    }
    println!("{}", Ipv4Addr::from(conn.addr));
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
    content_size = get_header::<usize>(&headers, "content-length").or(content_size);
    if let (Some(loc), Some(body_size)) = (body_loc, content_size) {
        println!("loc {} {} {}", loc, body_size, pld.len());
        if pld.len() - loc == body_size {
            println!("complete pld");
            let parsed: serde_json::Value = serde_json::from_slice(&pld[loc..]).unwrap();
            let parsed = parsed.get("test").unwrap();
            println!("val: {}", parsed.to_string());
        } else {
            println!("incomplete pld");
            resp_pool.insert(*conn, (body_loc, content_size, pld));
        }
    } else {
        resp_pool.insert(*conn, (body_loc, content_size, pld));
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

#[test]
fn process_packet_http_single() {
    let resp_pool: ConnMap = Arc::new(DashMap::new());
    let s = "HTTP/1.0 200 OK
Server: SimpleHTTP/0.6 Python/3.9.7
Date: Sat, 11 Dec 2021 00:47:31 GMT
Content-type: application/json; charset=utf-8
Content-Length: 24

{
    \"test\":\"test2\"
}";
    let s = s.replace("\n", "\r\n");
    let buf = s.as_bytes();
    let conn = Conn {
        pld_loc: 0,
        addr: 0,
        port: 0,
        padding: 0,
        ack_seq: 0,
    };
    tokio_test::block_on(process_packet(resp_pool, buf, &conn));
}

#[test]
fn process_packet_http_multi_json() {
    let resp_pool: ConnMap = Arc::new(DashMap::new());
    let s = "HTTP/1.0 200 OK
Server: SimpleHTTP/0.6 Python/3.9.7
Date: Sat, 11 Dec 2021 00:47:31 GMT
Content-type: application/json; charset=utf-8
Content-Length: 24

";
    let s = s.replace("\n", "\r\n");
    let buf = s.as_bytes();
    let conn = Conn {
        pld_loc: 0,
        addr: 0,
        port: 0,
        padding: 0,
        ack_seq: 0,
    };
    {
        let resp_pool = resp_pool.clone();
        tokio_test::block_on(process_packet(resp_pool, buf, &conn));
    }
    let s = "{
    \"test\":\"test2\"
}";
    let s = s.replace("\n", "\r\n");
    let buf = s.as_bytes();
    {
        let resp_pool = resp_pool.clone();
        tokio_test::block_on(process_packet(resp_pool, buf, &conn));
    }
}
