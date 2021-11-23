use plain::as_bytes;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

use anyhow::{bail, Result};
use libbpf_rs::MapFlags;
use structopt::StructOpt;

#[path = "./bpf/.output/xdp_parser.skel.rs"]
mod xdp_parser;
use xdp_parser::*;

#[derive(Debug, StructOpt)]
struct Command {
    /// Interface index to attach XDP program
    #[structopt(default_value = "0")]
    ifindex: i32,
}

fn bump_memlock_rlimit() -> Result<()> {
    let rlimit = libc::rlimit {
        rlim_cur: 128 << 20,
        rlim_max: 128 << 20,
    };

    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        bail!("Failed to increase rlimit");
    }

    Ok(())
}

fn main() -> Result<()> {
    let opts = Command::from_args();

    bump_memlock_rlimit()?;

    let skel_builder = XdpParserSkelBuilder::default();
    let open_skel = skel_builder.open()?;
    let mut skel = open_skel.load()?;
    let handle_egress_packet = skel
        .progs_mut()
        .handle_egress_packet()
        .attach_xdp(opts.ifindex)?;
    let handle_json = skel.progs_mut().handle_json().attach_xdp(opts.ifindex)?;
    skel.maps_mut().pdt_prog_map().update(
        &vec![0],
        unsafe { as_bytes(&handle_json.get_fd()) },
        MapFlags::empty(),
    )?;
    skel.links = XdpParserLinks {
        handle_json: Some(handle_json),
        handle_egress_packet: Some(handle_egress_packet),
    };

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        eprint!(".");
        thread::sleep(time::Duration::from_secs(1));
    }

    Ok(())
}
