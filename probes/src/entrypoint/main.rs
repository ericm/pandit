
#![no_std]
#![no_main]
use cty::*;

use redbpf_probes::xdp::prelude::*;

// Use the types you're going to share with userspace, eg:
// use probes::entrypoint::SomeEvent;

program!(0xFFFFFFFE, "GPL");

// The maps and probe functions go here, eg:
//
// #[map("syscall_events")]
// static mut syscall_events: PerfMap<SomeEvent> = PerfMap::with_max_entries(1024);
//
// #[kprobe("__x64_sys_open")]
// fn syscall_enter(regs: Registers) {
//   let pid_tgid = bpf_get_current_pid_tgid();
//   ...
//
//   let event = SomeEvent {
//     pid: pid_tgid >> 32,
//     ...
//   };
//   unsafe { syscall_events.insert(regs.ctx, &event) };
// }
