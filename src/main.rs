use futures::stream::StreamExt;
use std::{ffi::CStr, ptr};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use redbpf::load::Loader;
use redbpf::xdp;

use probes::entrypoint::Response;

fn probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/bpf/programs/entrypoint/entrypoint.elf"
    ))
}

const PIN_FILE: &str = "/sys/fs/bpf/requests";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mut loaded = Loader::load(probe_code()).expect("error on Loader::load");
    loaded
        .map_mut("REQUESTS")
        .expect("map not found")
        .pin(PIN_FILE)
        .expect("error on pinning");

    loaded
        .xdp_mut("entrypoint")
        .expect("error on Loaded::entrypoint")
        .attach_xdp("lo", xdp::Flags::default())
        .expect("error on XDP::attach_xdp");

    while let Some((map_name, events)) = loaded.events.next().await {
        if map_name == "REQUESTS" {
            for event in events {
                let resp = unsafe { ptr::read(event.as_ptr() as *const Response) };
                println!("{}", resp.tuple);
            }
        }
    }
}
