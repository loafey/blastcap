use std::net::SocketAddr;

use crate::network::{
    ClientPoll, HostPoll, MetadataExt, NetworkClientExt, NetworkHostExt,
    messages::{ClientRequest, ServerMessage},
};
use async_trait::async_trait;
use steamworks::{Client, SteamId};

pub(super) struct SteamClient {}
#[async_trait]
impl NetworkClientExt for SteamClient {
    async fn poll(&mut self) -> anyhow::Result<ClientPoll> {
        todo!()
    }
    async fn send(&mut self, req: ClientRequest) -> anyhow::Result<()> {
        todo!()
    }
}
pub(super) struct SteamHost {}

#[async_trait]
impl NetworkHostExt for SteamHost {
    async fn mock(&mut self, req: ClientRequest) -> anyhow::Result<()> {
        todo!()
    }
    async fn poll(&mut self) -> anyhow::Result<HostPoll> {
        todo!()
    }

    async fn send(&mut self, addr: SocketAddr, req: ServerMessage) -> anyhow::Result<()> {
        todo!()
    }

    async fn broadcast(&mut self, req: ServerMessage) -> anyhow::Result<()> {
        todo!()
    }

    fn remove_client(&mut self, addr: SocketAddr) {
        todo!()
    }

    fn get_clients(&self) -> Vec<SocketAddr> {
        todo!()
    }

    fn get_client_count(&self) -> u32 {
        todo!()
    }
}

pub(super) struct SteamMetadata {
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
}
