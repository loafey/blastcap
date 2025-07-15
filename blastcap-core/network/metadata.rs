use crate::network::impls::{steam::SteamMetadata, tcp::TcpMetadata};
use async_trait::async_trait;
use std::{fmt::Debug, ops::Deref, sync::LazyLock};
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender, channel},
    oneshot::channel as oneshot,
};

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
        let inner: Box<dyn MetadataExt + Send + Sync + 'static> = match super::use_tcp() {
            true => Box::new(TcpMetadata::new()),
            false => Box::new(SteamMetadata::new().unwrap()),
        };
        let mut lock = META_DATA.blocking_lock();
        *lock = Some(Ok(Metadata { inner }));
    }
    pub async fn grab_host() -> (Self, Receiver<MetadataTask>) {
        let mut lock = META_DATA.lock().await;
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
    fn create_lobby(&self) -> anyhow::Result<u64>;
    async fn tick(&self) -> anyhow::Result<()>;
}
