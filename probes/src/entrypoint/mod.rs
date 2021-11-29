#[derive(Copy, Clone)]
#[repr(C)]
pub struct SocketAddr {
    pub addr: u32,
    pub port: u16,
    _padding: u16,
}

impl SocketAddr {
    pub fn new(addr: u32, port: u16) -> Self {
        SocketAddr {
            addr,
            port,
            _padding: 0,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Conn {
    pub pld_loc: u32,
}

impl Conn {
    pub fn new(pld_loc: u32) -> Self {
        Conn { pld_loc }
    }
}
