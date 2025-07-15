use crate::network::{
    ClientPoll, HostPoll, MetadataExt, NetworkClientExt, NetworkHost, NetworkHostExt,
    messages::{ClientRequest, ServerMessage},
    metadata, tick,
};
use async_trait::async_trait;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use steamworks::{
    Client, LobbyCreated, LobbyEnter, SteamId, networking_sockets::ListenSocket,
    networking_types::NetworkingConfigEntry,
};
use tokio::sync::{mpsc, oneshot};

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
        todo!("client poll")
    }
    async fn send(&mut self, _req: ClientRequest) -> anyhow::Result<()> {
        todo!("client send")
    }
}
pub struct SteamHost {
    lobby_id: u64,
    kill_send: oneshot::Sender<()>,
    msg_recv: mpsc::Receiver<()>,
}

#[async_trait]
impl NetworkHostExt for SteamHost {
    async fn mock(&mut self, _req: ClientRequest) -> anyhow::Result<()> {
        todo!("mock")
    }
    async fn poll(&mut self) -> anyhow::Result<HostPoll> {
        self.listen_socket.events();
        tokio::select! {
            _ = tick() => Ok(HostPoll::Tick)
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
    pub fn new() -> Self {
        let client = steamworks::Client::init_app(480).unwrap();
        Self { client }
    }
}

#[async_trait]
impl MetadataExt for SteamMetadata {
    fn get_my_id(&self) -> u64 {
        self.client.user().steam_id().raw()
    }

    fn register_callbacks(&self) {}

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

    async fn create_lobby(&self) -> anyhow::Result<NetworkHost> {
        let (send, recv) = std::sync::mpsc::channel();
        self.client
            .matchmaking()
            .create_lobby(steamworks::LobbyType::FriendsOnly, 8, move |r| {
                let Err(e) = send.send(r) else { return };
                panic!("{e}");
            });
        let listen_socket = self.client.networking_sockets().create_listen_socket_ip(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8000)),
            [],
        )?;
        let lobby_id = loop {
            self.client.run_callbacks();
            if let Ok(id) = recv.try_recv() {
                break id?.raw();
            }
        };
        let (kill_send, kill_recv) = oneshot::channel();
        let (msg_send, msg_recv) = mpsc::channel(1000);
        std::thread::spawn(move || {
            loop {
                let ev = listen_socket.receive_event();
            }
        });
        Ok(NetworkHost::new(SteamHost {
            lobby_id,
            kill_send,
            msg_recv,
        }))
    }
}
