use crate::network::{
    ClientPoll, HostPoll, Metadata, MetadataExt, MetadataTask, NetworkClientExt, NetworkHostExt,
    TICK_RATE,
    messages::{ClientRequest, ServerMessage},
};
use async_trait::async_trait;
use std::net::SocketAddr;
use steamworks::{Client, LobbyCreated, LobbyEnter, SteamId};
use tokio::sync::mpsc::Receiver;

pub struct SteamClient {}
impl SteamClient {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn NetworkClientExt> {
        Box::new(SteamClient {})
    }
}
#[async_trait]
impl NetworkClientExt for SteamClient {
    async fn poll(&mut self) -> anyhow::Result<ClientPoll> {
        todo!()
    }
    async fn send(&mut self, _req: ClientRequest) -> anyhow::Result<()> {
        todo!()
    }
}
pub struct SteamHost {
    metadata: Metadata,
    metadata_recv: Receiver<MetadataTask>,
    _lobby_id: u64,
}
impl SteamHost {
    pub async fn new() -> anyhow::Result<Self> {
        let (metadata, metadata_recv) = Metadata::grab_host().await;
        let lobby_id = metadata.create_lobby()?;

        Ok(Self {
            metadata,
            metadata_recv,
            _lobby_id: lobby_id,
        })
    }
}

#[async_trait]
impl NetworkHostExt for SteamHost {
    async fn mock(&mut self, _req: ClientRequest) -> anyhow::Result<()> {
        todo!("mock")
    }
    async fn poll(&mut self) -> anyhow::Result<HostPoll> {
        self.metadata.tick().await?;
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs_f64(const { 1.0 / TICK_RATE as f64 })) => {
                while let Ok(task) = self.metadata_recv.try_recv() {
                    task(&self.metadata)?;
                }
                Ok(HostPoll::Tick)
            }
        }
    }

    async fn send(&mut self, _addr: SocketAddr, _req: ServerMessage) -> anyhow::Result<()> {
        todo!("send")
    }

    async fn broadcast(&mut self, _req: ServerMessage) -> anyhow::Result<()> {
        todo!("broadcast")
    }

    fn remove_client(&mut self, _addr: SocketAddr) {
        todo!("remove_client")
    }

    fn get_clients(&self) -> Vec<SocketAddr> {
        todo!("get_clients")
    }

    fn get_client_count(&self) -> u32 {
        todo!("get_client_count")
    }
}

pub struct SteamMetadata {
    client: Client,
}
impl SteamMetadata {
    pub fn new() -> anyhow::Result<Self> {
        let client = steamworks::Client::init_app(480)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl MetadataExt for SteamMetadata {
    fn get_my_id(&self) -> u64 {
        self.client.user().steam_id().raw()
    }

    fn get_name(&self, id: u64) -> anyhow::Result<String> {
        Ok(self
            .client
            .friends()
            .get_friend(SteamId::from_raw(id))
            .name())
    }
    async fn tick(&self) -> anyhow::Result<()> {
        self.client.run_callbacks();
        Ok(())
    }

    fn get_avatar(&self, id: u64) -> Option<(Vec<u8>, u16, u16)> {
        self.client
            .friends()
            .get_friend(SteamId::from_raw(id))
            .medium_avatar()
            .map(|a| (a, 64, 64))
    }

    fn create_lobby(&self) -> anyhow::Result<u64> {
        self.client
            .register_callback(|p: LobbyEnter| println!("---- {p:?}"));
        self.client
            .register_callback(|p: LobbyCreated| println!("---- {p:?}"));
        let (send, recv) = std::sync::mpsc::channel();
        self.client
            .matchmaking()
            .create_lobby(steamworks::LobbyType::FriendsOnly, 8, move |r| {
                let Err(e) = send.send(r) else { return };
                panic!("{e}");
            });
        let lobby_id = loop {
            self.client.run_callbacks();
            if let Ok(id) = recv.try_recv() {
                break id?.raw();
            }
        };
        Ok(lobby_id)
    }
}
