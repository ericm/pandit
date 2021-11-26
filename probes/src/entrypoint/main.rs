#![no_std]
#![no_main]
use cty::*;

use probes::entrypoint::Response;
use probes::gen_bindings::{iphdr, tcphdr};
use redbpf_probes::xdp::prelude::*;

// Use the types you're going to share with userspace, eg:
// use probes::entrypoint::SomeEvent;

program!(0xFFFFFFFE, "GPL");

// The maps and probe functions go here, eg:
//
#[map]
static mut CONN_MAP: HashMap<u64, Response> = HashMap::with_max_entries(1024);

#[xdp]
pub extern "C" fn entrypoint(ctx: XdpContext) -> XdpResult {
    let ip = ctx.ip()? as *const iphdr;
    let transport = match ctx.transport()? {
        Transport::TCP(hdr) => hdr as *const tcphdr,
        _ => return Ok(XdpAction::Pass),
    };
    let data = ctx.data()?;
    let resp = Response::default();
    unsafe {
        let key = u64::from((*ip).daddr) << 32 | u64::from((*transport).dest) << 16;
        CONN_MAP.set(&key, &resp);
    }
    let buf: [u8; 7] = match data.read() {
        Some(b) => b,
        None => return XdpAction::Pass,
    };

    match buf {
        b"HTTP/1.0" => None,
        _ => return XdpAction::Pass,
    }

    Ok(XdpAction::Pass)
}
