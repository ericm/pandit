#![no_std]
#![no_main]
use cty::*;

use probes::gen_bindings::*;

use probes::entrypoint::Response;
use redbpf_probes::xdp::prelude::*;

// Use the types you're going to share with userspace, eg:
// use probes::entrypoint::SomeEvent;

program!(0xFFFFFFFE, "GPL");

// The maps and probe functions go here, eg:
//
#[map(link_section = "maps/conns")]
static mut CONN_MAP: HashMap<u64, Response> = HashMap::with_max_entries(8192);

#[xdp]
pub extern "C" fn entrypoint(ctx: XdpContext) -> XdpResult {
    let ip = ctx.ip()? as *const iphdr;
    let resp = Response::default();
    // let s: &[u32] = "sdsa";
    unsafe {
        let key = u64::from((*ip).daddr);
        CONN_MAP.set(&key, &resp);
        bpf_trace_printk(b"asdas");
    }
    Ok(XdpAction::Pass)
}
