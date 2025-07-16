use std::net::SocketAddr;

use crate::network::{BOT_ADDR, HOST_ADDR};

pub trait SocketAddrExt {
    fn is_bot(&self) -> bool;
    fn is_host(&self) -> bool;
}
impl SocketAddrExt for SocketAddr {
    fn is_bot(&self) -> bool {
        *self == *BOT_ADDR
    }
    fn is_host(&self) -> bool {
        *self == *HOST_ADDR
    }
}
