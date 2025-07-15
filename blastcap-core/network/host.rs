use crate::network::messages::{ClientRequest, ServerMessage};
use async_trait::async_trait;
use std::{
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

pub struct NetworkHost {
    inner: Box<dyn NetworkHostExt>,
}
impl NetworkHost {
    pub fn new<T: NetworkHostExt + 'static>(t: T) -> Self {
        NetworkHost { inner: Box::new(t) }
    }
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
