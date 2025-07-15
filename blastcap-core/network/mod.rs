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

static LOCAL_ADDR: LazyLock<SocketAddr> = LazyLock::new(|| "0.0.0.0:0".parse().unwrap());

use crate::network::{
    impls::{
        steam::{SteamHost, SteamMetadata},
        tcp::{TcpClient, TcpHost, TcpMetadata},
    },
    messages::{ClientRequest, ServerMessage},
};
use async_trait::async_trait;
pub use socket_addr_ext::*;
use std::{
    fmt::Debug,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};
use tokio::{
    net::ToSocketAddrs,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
        oneshot::channel as oneshot,
    },
};

fn use_tcp() -> bool {
    std::env::var("BLASTCAP_USE_TCP").is_ok()
}

pub const TICK_RATE: usize = 30;
