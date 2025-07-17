pub mod channel;
mod client;
pub use client::*;
mod host;
pub use host::*;
mod impls;
pub mod messages;
mod metadata;
pub use metadata::*;
mod socket_addr_ext;

static BOT_ADDR: u64 = u64::from_be_bytes([0, 0, 0, 0, 0, 1, 0, 0]);
static HOST_ADDR: u64 = 0;

use crate::network::messages::{ClientRequest, ServerMessage};
pub use socket_addr_ext::*;

pub const TICK_RATE: usize = 30;
pub async fn tick() {
    tokio::time::sleep(std::time::Duration::from_secs_f64(
        const { 1.0 / TICK_RATE as f64 },
    ))
    .await
}
