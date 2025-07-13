pub mod channel;
pub mod messages;
mod socket_addr_ext;
mod steam;
mod tcp;

static LOCAL_ADDR: LazyLock<SocketAddr> = LazyLock::new(|| "0.0.0.0:0".parse().unwrap());

use crate::network::{
    messages::{ClientRequest, ServerMessage},
    steam::SteamMetadata,
    tcp::{TcpClient, TcpHost, TcpMetadata},
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
    async fn tcp<A: ToSocketAddrs>(addr: A) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Box::new(TcpClient::new(addr).await?),
        })
    }

    pub async fn create(addr: String) -> anyhow::Result<Self> {
        match use_tcp() {
            true => Self::tcp(format!("{addr}:8000")).await,
            false => todo!("steam client"),
        }
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
    async fn tcp(port: u16) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Box::new(TcpHost::new(port).await?),
        })
    }
    pub async fn create() -> anyhow::Result<Self> {
        match use_tcp() {
            true => Self::tcp(8000).await,
            false => todo!("steam host"),
        }
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

#[allow(clippy::type_complexity)]
static META_DATA: LazyLock<Mutex<Option<Result<Metadata, Sender<MetadataTask>>>>> =
    LazyLock::new(Default::default);
#[allow(clippy::type_complexity)]
pub enum MetadataHolder {
    Owned(Metadata),
    NotOwned(Sender<MetadataTask>),
}
impl Debug for MetadataHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Owned(_) => write!(f, "Owned"),
            Self::NotOwned(_) => write!(f, "NotOwned"),
        }
    }
}
impl MetadataHolder {
    pub async fn act<
        T: 'static + Send,
        F: FnOnce(&Metadata) -> anyhow::Result<T> + 'static + Send,
    >(
        &self,
        act: F,
    ) -> anyhow::Result<T> {
        match self {
            MetadataHolder::Owned(metadata) => act(metadata),
            MetadataHolder::NotOwned(sender) => {
                let (send, recv) = oneshot();
                _ = sender
                    .send(Box::new(move |m| {
                        _ = send.send(act(m));
                        Ok(())
                    }))
                    .await;
                recv.await?
            }
        }
    }
}

pub type MetadataTask = Box<dyn FnOnce(&Metadata) -> anyhow::Result<()> + Send>;
pub struct Metadata {
    inner: Box<dyn MetadataExt + 'static + Send + Sync>,
}
impl Metadata {
    pub unsafe fn peek() -> Option<&'static Metadata> {
        match &*META_DATA.blocking_lock() {
            Some(Ok(m)) => Some(unsafe { std::mem::transmute::<&Metadata, &'static Metadata>(m) }),
            None | Some(Err(_)) => None,
        }
    }
    pub fn init() {
        let inner: Box<dyn MetadataExt + Send + Sync + 'static> = match use_tcp() {
            true => Box::new(TcpMetadata::new()),
            false => Box::new(SteamMetadata::new().unwrap()),
        };
        let mut lock = META_DATA.blocking_lock();
        *lock = Some(Ok(Metadata { inner }));
    }
    pub fn grab_host() -> (Self, Receiver<MetadataTask>) {
        let mut lock = META_DATA.blocking_lock();
        match &*lock {
            Some(i) => match i {
                Ok(_) => {
                    let (send, recv) = channel(1);
                    let mut wrapped = Some(Err(send));
                    std::mem::swap(&mut wrapped, &mut *lock);
                    (wrapped.unwrap().unwrap(), recv)
                }
                Err(_) => panic!("metadata has already been grabbed"),
            },
            None => panic!("metadata has not been initialized"),
        }
    }
    pub async fn grab_client() -> MetadataHolder {
        let mut lock = META_DATA.lock().await;
        match &*lock {
            Some(_) => {
                let mut wrapped = None;
                std::mem::swap(&mut wrapped, &mut *lock);
                match wrapped.unwrap() {
                    Ok(o) => MetadataHolder::Owned(o),
                    Err(o) => MetadataHolder::NotOwned(o),
                }
            }
            None => panic!("metadata has not been initialized"),
        }
    }
}
impl Deref for Metadata {
    type Target = dyn MetadataExt;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
#[async_trait]
pub trait MetadataExt {
    fn get_my_id(&self) -> u64;
    fn get_name(&self, id: u64) -> anyhow::Result<String>;
    fn get_avatar(&self, id: u64) -> Option<(Vec<u8>, u16, u16)>;
    async fn tick(&self) -> anyhow::Result<()>;
}
