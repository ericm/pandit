use cty::*;

// This is where you should define the types shared by the kernel and user
// space, eg:
//
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Response {
    pub tuple: u64,
}

impl Default for Response {
    fn default() -> Self {
        Response { tuple: 0 }
    }
}
