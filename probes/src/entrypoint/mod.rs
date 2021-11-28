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
