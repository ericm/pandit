use futures::stream::StreamExt;
use std::{ffi::CStr, ptr};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use redbpf::load::Loader;

use probes::openmonitor::OpenPath;

fn probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/bpf/programs/openmonitor/openmonitor.elf"
    ))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mut loaded = Loader::load(probe_code()).expect("error on Loader::load");

    loaded
        .kprobe_mut("do_sys_openat2")
        .expect("error on Loaded::kprobe_mut")
        .attach_kprobe("do_sys_openat2", 0)
        .expect("error on KProbe::attach_kprobe");

    while let Some((map_name, events)) = loaded.events.next().await {
        if map_name == "OPEN_PATHS" {
            for event in events {
                let open_path = unsafe { ptr::read(event.as_ptr() as *const OpenPath) };
                unsafe {
                    let cfilename = CStr::from_ptr(open_path.filename.as_ptr() as *const _);
                    println!("{}", cfilename.to_string_lossy());
                };
            }
        }
    }
}
