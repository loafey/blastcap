use crate::network::{
    impls::{steam::SteamClient, tcp::TcpClient},
    messages::{ClientRequest, ServerMessage},
};
use async_trait::async_trait;
use std::ops::{Deref, DerefMut};

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
    pub fn new<T: NetworkClientExt + 'static>(t: T) -> Self {
        Self { inner: Box::new(t) }
    }
    // pub async fn create(addr: String) -> anyhow::Result<Self> {
    //     Ok(Self {
    //         inner: match super::use_tcp() {
    //             true => TcpClient::new(format!("{addr}:8000")).await?,
    //             false => SteamClient::new(),
    //         },
    //     })
    // }
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
