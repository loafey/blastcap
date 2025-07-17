use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::network::{BOT_ADDR, HOST_ADDR};

pub trait SocketAddrExt {
    fn raw(&self) -> u64;
    fn from_raw(input: u64) -> Self;
}
impl SocketAddrExt for SocketAddr {
    fn raw(&self) -> u64 {
        match self {
            SocketAddr::V4(addr) => {
                let port: [u8; 2] = addr.port().to_be_bytes();
                let addr: [u8; 4] = addr.ip().to_bits().to_be_bytes();
                u64::from_be_bytes([addr[0], addr[1], addr[2], addr[3], port[0], port[1], 0, 0])
            }
            SocketAddr::V6(_) => panic!("IPv6 is not supported"),
        }
    }
    fn from_raw(input: u64) -> Self {
        let [ip1, ip2, ip3, ip4, p1, p2, _, _] = input.to_be_bytes();
        SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::from_bits(u32::from_be_bytes([ip1, ip2, ip3, ip4])),
            u16::from_be_bytes([p1, p2]),
        ))
    }
}
pub trait IdentityExt {
    fn is_bot(&self) -> bool;
    fn is_host(&self) -> bool;
}
impl IdentityExt for u64 {
    fn is_bot(&self) -> bool {
        *self == BOT_ADDR
    }
    fn is_host(&self) -> bool {
        *self == HOST_ADDR
    }
}
