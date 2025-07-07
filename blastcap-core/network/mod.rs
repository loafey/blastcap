pub mod channel;
pub mod messages;
mod socket_addr_ext;
mod tcp;
pub use socket_addr_ext::*;

static LOCAL_ADDR: LazyLock<SocketAddr> = LazyLock::new(|| "0.0.0.0:0".parse().unwrap());

use crate::network::{
    messages::{ClientRequest, ServerMessage},
    tcp::{TcpClient, TcpHost},
};
use async_trait::async_trait;
use std::{
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};
use tokio::net::ToSocketAddrs;

pub const TICK_RATE: usize = 30;

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
    pub async fn tcp<A: ToSocketAddrs>(addr: A) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Box::new(TcpClient::new(addr).await?),
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

pub struct NetworkHost {
    inner: Box<dyn NetworkHostExt>,
}
unsafe impl Send for NetworkHost {}
impl Deref for NetworkHost {
    type Target = dyn NetworkHostExt;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
impl DerefMut for NetworkHost {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}
impl NetworkHost {
    pub async fn tcp(port: u16) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Box::new(TcpHost::new(port).await?),
        })
    }
}

pub enum HostPoll {
    RemoveClient(SocketAddr),
    ClientConnected(SocketAddr),
    ClientRequest {
        addr: SocketAddr,
        req: ClientRequest,
    },
    Tick,
}

#[async_trait]
pub trait NetworkHostExt {
    async fn poll(&mut self) -> anyhow::Result<HostPoll>;
    async fn mock(&mut self, msg: ClientRequest) -> anyhow::Result<()>;
    async fn send(&mut self, addr: SocketAddr, req: ServerMessage) -> anyhow::Result<()>;
    async fn broadcast(&mut self, req: ServerMessage) -> anyhow::Result<()>;
    fn remove_client(&mut self, addr: SocketAddr);
    fn get_clients(&self) -> Vec<SocketAddr>;
    fn get_client_count(&self) -> u32;
}
