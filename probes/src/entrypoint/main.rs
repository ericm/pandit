#![no_std]
#![no_main]
use cty::*;

use probes::entrypoint::Response;
use redbpf_probes::bpf_iter::prelude::*;
use redbpf_probes::xdp::prelude::*;

// Use the types you're going to share with userspace, eg:
// use probes::entrypoint::SomeEvent;

program!(0xFFFFFFFE, "GPL");

// The maps and probe functions go here, eg:
//
// #[map]
// static mut REQUESTS: HashMap<u32, Response> = HashMap::with_max_entries(1024);

#[xdp]
fn entrypoint(ctx: XdpContext) -> XdpResult {
    bpf_trace_printk(b"packet received");
    let ip = ctx.ip()?;
    let transport = match ctx.transport()? {
        Transport::TCP(hdr) => hdr,
        _ => return Ok(XdpAction::Pass),
    };
    let data = ctx.data()?;
    let resp = Response::default();
    unsafe {
        // let key = u64::from((*ip).daddr) << 32 | u64::from((*transport).dest) << 16;
        let key = (*ip).daddr;
        // REQUESTS.set(&key, &resp);
    }
    let buf: [u8; 8] = match data.read() {
        Ok(b) => b,
        Err(_) => return Ok(XdpAction::Pass),
    };

    match &buf {
        b"HTTP/1.0" => (),
        _ => return Ok(XdpAction::Pass),
    };

    let http_pld = match data.slice(1500 - data.offset()) {
        Ok(b) => b,
        Err(_) => return Ok(XdpAction::Pass),
    };

    let bf: &mut [u8] = &mut [];
    bf.copy_from_slice(http_pld);

    for b in bf {
        bpf_trace_printk(&[*b]);
    }
    Ok(XdpAction::Pass)
}
