#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Conn {
    pub pld_loc: u32,
    pub addr: u32,
    pub port: u16,
    pub ack_seq: u32,
    pub padding: u16,
}
