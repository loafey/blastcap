use crate::network::{
    impls::{
        steam::{SteamClient, SteamHost, SteamMetadata},
        tcp::{TcpClient, TcpHost, TcpMetadata},
    },
    messages::{ClientRequest, ServerMessage},
    socket_addr_ext::*,
};
use async_trait::async_trait;
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

pub struct NetworkClient {
    inner: Box<dyn NetworkClientExt>,
}
unsafe impl Send for NetworkClient {}
impl Deref for NetworkClient {
    type Target = dyn NetworkClientExt;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
impl DerefMut for NetworkClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}
impl NetworkClient {
    pub async fn create(addr: String) -> anyhow::Result<Self> {
        Ok(Self {
            inner: match super::use_tcp() {
                true => TcpClient::new(format!("{addr}:8000")).await?,
                false => SteamClient::new(),
            },
        })
    }
}

pub enum ClientPoll {
    Message(ServerMessage),
    Tick,
}

#[async_trait]
pub trait NetworkClientExt {
    async fn poll(&mut self) -> anyhow::Result<ClientPoll>;
    async fn send(&mut self, req: ClientRequest) -> anyhow::Result<()>;
}
