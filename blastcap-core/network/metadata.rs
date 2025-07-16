use async_trait::async_trait;
use std::{ops::Deref, pin::Pin, sync::LazyLock};
use tokio::sync::{
    mpsc::{Sender, channel},
    oneshot::channel as oneshot,
};

use crate::network::{
    NetworkClient, NetworkHost,
    impls::{steam::SteamMetadata, tcp::TcpMetadata},
    tick, use_tcp,
};

#[allow(clippy::type_complexity)]
static METADATA: LazyLock<Sender<MetadataTask>> = LazyLock::new(|| {
    let (send, mut recv) = channel::<MetadataTask>(10);
    let m = Metadata {
        inner: match use_tcp() {
            true => Box::new(TcpMetadata::new()),
            false => Box::new(SteamMetadata::new()),
        },
    };
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Unable to create Runtime");
        let _enter = rt.enter();
        rt.block_on(async {
            loop {
                tokio::select! {
                    msg = recv.recv() => {
                        let Some(act) = msg else { break };
                        let Err(e) = act(unsafe{std::mem::transmute::<&Metadata, &'static Metadata>(&m)}).await else { continue };
                        panic!("metadata panic: {e}")
                    }
                    _ = tick() => {
                        let Err(e) = m.tick().await else { continue };
                        panic!("metadata tick panic: {e}")
                    }
                }
            }
        });
        panic!("metadata early exit");
    });

    send
});

// #[track_caller]
pub async fn metadata<
    T: 'static + Send,
    I: FnOnce(&'static Metadata) -> F + Send + 'static,
    F: Future<Output = T> + Send,
>(
    f: I,
) -> T {
    let (send, recv) = oneshot();
    METADATA
        .send(Box::new(move |m| {
            Box::pin(async move {
                _ = send.send(f(m).await);
                Ok(())
            })
        }))
        .await
        .unwrap();
    recv.await.unwrap()
}

#[track_caller]
pub fn metadata_block<
    T: 'static + Send,
    I: FnOnce(&'static Metadata) -> F + Send + 'static,
    F: Future<Output = T> + Send,
>(
    f: I,
) -> T {
    let (send, recv) = oneshot();
    METADATA
        .blocking_send(Box::new(move |m| {
            Box::pin(async move {
                _ = send.send(f(m).await);
                Ok(())
            })
        }))
        .unwrap();
    recv.blocking_recv().unwrap()
}

pub type MetadataTask =
    Box<dyn FnOnce(&'static Metadata) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>> + Send>;
pub struct Metadata {
    inner: Box<dyn MetadataExt + 'static + Send + Sync>,
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
    fn register_callbacks(&self);
    async fn create_lobby(&self) -> anyhow::Result<NetworkHost>;
    async fn create_client(&self, lobby: u64) -> anyhow::Result<NetworkClient>;
    async fn tick(&self) -> anyhow::Result<()>;
}
