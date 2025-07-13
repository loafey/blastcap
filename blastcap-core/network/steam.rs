use std::net::SocketAddr;

use crate::network::{
    ClientPoll, HostPoll, NetworkClientExt, NetworkHostExt,
    messages::{ClientRequest, ServerMessage},
};
use async_trait::async_trait;

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
