use std::net::SocketAddr;

use crate::network::LOCAL_ADDR;

pub trait SocketAddrExt {
    fn is_host(&self) -> bool;
}
impl SocketAddrExt for SocketAddr {
    fn is_host(&self) -> bool {
        *self == *LOCAL_ADDR
    }
}
