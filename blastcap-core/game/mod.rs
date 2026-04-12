use crate::network::NetworkHost;
use std::{mem::MaybeUninit, time::Instant};

mod actor;
mod map;
pub mod state;

#[derive(Default)]
pub struct ServerData {
    host_player: Option<u64>,
    tick: usize,
}

pub struct Arg<'l> {
    pub data: &'l mut ServerData,
    pub host: &'l mut NetworkHost,
    pub last_tick: &'l mut Instant,
}
impl<'l> Arg<'l> {
    // TODO: Don't know what I was thinking when adding this function... this should be removed.
    pub unsafe fn clone(&self) -> Self {
        unsafe {
            let new = MaybeUninit::uninit();
            let ptr = new.as_ptr() as *mut Self;
            std::ptr::copy(self, ptr, 1);

            new.assume_init()
        }
    }
}
