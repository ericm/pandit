// This program can be executed by
// # cargo run --example tcp-lifetime [interface]
#![no_std]
#![no_main]
use core::{
    convert::TryInto,
    mem::{self, MaybeUninit},
};
use memoffset::offset_of;

use redbpf_probes::socket_filter::prelude::*;

use probes::entrypoint::*;

#[map(link_section = "maps/established")]
static mut ESTABLISHED: PerfMap<Conn> = PerfMap::with_max_entries(10240);

program!(0xFFFFFFFE, "GPL");
#[socket_filter]
fn measure_tcp_lifetime(skb: SkBuff) -> SkBuffResult {
    bpf_trace_printk(b"input");
    let eth_len = mem::size_of::<ethhdr>();
    let eth_proto = skb.load::<__be16>(offset_of!(ethhdr, h_proto))? as u32;
    if eth_proto != ETH_P_IP {
        return Ok(SkBuffAction::Ignore);
    }

    let ip_proto = skb.load::<__u8>(eth_len + offset_of!(iphdr, protocol))? as u32;
    if ip_proto != IPPROTO_TCP {
        return Ok(SkBuffAction::Ignore);
    }

    let mut ip_hdr = unsafe { MaybeUninit::<iphdr>::zeroed().assume_init() };
    ip_hdr._bitfield_1 = __BindgenBitfieldUnit::new([skb.load::<u8>(eth_len)?]);
    if ip_hdr.version() != 4 {
        return Ok(SkBuffAction::Ignore);
    }

    let ihl = ip_hdr.ihl() as usize;
    let dst = SocketAddr::new(
        skb.load::<__be32>(eth_len + offset_of!(iphdr, daddr))?,
        skb.load::<__be16>(eth_len + ihl * 4 + offset_of!(tcphdr, dest))?,
    );
    let mut tcp_hdr = unsafe { MaybeUninit::<tcphdr>::zeroed().assume_init() };
    tcp_hdr._bitfield_1 = __BindgenBitfieldUnit::new([
        skb.load::<u8>(eth_len + ihl * 4 + offset_of!(tcphdr, _bitfield_1))?,
        skb.load::<u8>(eth_len + ihl * 4 + offset_of!(tcphdr, _bitfield_1) + 1)?,
    ]);

    let doff: usize = (tcp_hdr.doff() * 4).into();

    let mut http_hdr = unsafe { MaybeUninit::<[u8; 8]>::zeroed().assume_init() };
    let off: i32 = (eth_len + ihl * 4 + doff).try_into().unwrap();
    http_hdr = [
        skb.load::<u8>(eth_len + ihl * 4 + doff)?,
        skb.load::<u8>(eth_len + ihl * 4 + doff + 1)?,
        skb.load::<u8>(eth_len + ihl * 4 + doff + 2)?,
        skb.load::<u8>(eth_len + ihl * 4 + doff + 3)?,
        skb.load::<u8>(eth_len + ihl * 4 + doff + 4)?,
        skb.load::<u8>(eth_len + ihl * 4 + doff + 5)?,
        skb.load::<u8>(eth_len + ihl * 4 + doff + 6)?,
        skb.load::<u8>(eth_len + ihl * 4 + doff + 7)?,
    ];

    match &http_hdr {
        b"HTTP/1.0" => (),
        _ => return Ok(SkBuffAction::Ignore),
    };
    let conn = Conn::new(off.try_into().unwrap());
    unsafe { ESTABLISHED.insert(skb.skb as *mut __sk_buff, &conn) };

    Ok(SkBuffAction::SendToUserspace)
}
