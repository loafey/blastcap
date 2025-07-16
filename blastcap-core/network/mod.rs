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

static BOT_ADDR: LazyLock<SocketAddr> = LazyLock::new(|| "0.0.0.0:1".parse().unwrap());
static HOST_ADDR: LazyLock<SocketAddr> = LazyLock::new(|| "0.0.0.0:0".parse().unwrap());

use crate::network::messages::{ClientRequest, ServerMessage};
pub use socket_addr_ext::*;
use std::{net::SocketAddr, sync::LazyLock};

fn use_tcp() -> bool {
    std::env::var("BLASTCAP_USE_TCP").is_ok()
}

pub const TICK_RATE: usize = 30;
pub async fn tick() {
    tokio::time::sleep(std::time::Duration::from_secs_f64(
        const { 1.0 / TICK_RATE as f64 },
    ))
    .await
}
