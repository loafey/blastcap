use crate::network::NetworkHost;
use std::{mem::MaybeUninit, net::SocketAddr};
use tokio::time::Instant;

mod actor;
mod map;
pub mod state;

#[derive(Default)]
pub struct ServerData {
    host_player: Option<SocketAddr>,
    tick: usize,
}

pub struct Arg<'l> {
    pub data: &'l mut ServerData,
    pub host: &'l mut NetworkHost,
    pub last_tick: &'l mut Instant,
}
impl<'l> Arg<'l> {
    pub unsafe fn clone(&self) -> Self {
        unsafe {
            let new = MaybeUninit::uninit();
            let ptr = new.as_ptr() as *mut Self;
            std::ptr::copy(self, ptr, 1);

            new.assume_init()
        }
    }
}
