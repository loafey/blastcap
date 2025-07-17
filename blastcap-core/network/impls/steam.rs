use crate::network::{
    ClientPoll, HOST_ADDR, HostPoll, MetadataExt, NetworkClient, NetworkClientExt, NetworkHost,
    NetworkHostExt,
    messages::{ClientRequest, ServerMessage},
    tick,
};
use async_trait::async_trait;
use std::net::SocketAddr;
use steamworks::{Client, SteamId, networking_types::ListenSocketEvent};
use tokio::sync::{mpsc, oneshot};

pub enum SteamClient {
    Real,
    Channel {
        read: mpsc::Receiver<ServerMessage>,
        write: mpsc::Sender<ClientRequest>,
    },
}
#[async_trait]
impl NetworkClientExt for SteamClient {
    async fn poll(&mut self) -> anyhow::Result<ClientPoll> {
        let fut = match self {
            SteamClient::Real => todo!("steam real poll"),
            SteamClient::Channel { read, .. } => read,
        };
        tokio::select! {
            msg = fut.recv() => {
                let Some(msg) = msg else { panic!("no clients somehow") };
                Ok(ClientPoll::Message(msg))
            }
            _ = tick() => {
                Ok(ClientPoll::Tick)
            }
        }
    }
    async fn send(&mut self, req: ClientRequest) -> anyhow::Result<()> {
        match self {
            SteamClient::Real => todo!("steam real send"),
            SteamClient::Channel { write, .. } => write.send(req).await?,
        };
        Ok(())
    }
}
pub struct SteamHost {
    lobby_id: u64,
    kill_send: oneshot::Sender<()>,
    msg_recv: mpsc::Receiver<()>,
    own_recv: mpsc::Receiver<ClientRequest>,
    own_send: mpsc::Sender<ServerMessage>,
}

#[async_trait]
impl NetworkHostExt for SteamHost {
    async fn mock(&mut self, _req: ClientRequest) -> anyhow::Result<()> {
        todo!("mock")
    }
    async fn poll(&mut self) -> anyhow::Result<HostPoll> {
        // self.listen_socket.events();
        tokio::select! {
            own = self.own_recv.recv() => {
                let Some(req) = own else { unreachable!() };
                Ok(HostPoll::ClientRequest { addr: *HOST_ADDR, req })
            }
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
    own_client: Option<(mpsc::Receiver<ServerMessage>, mpsc::Sender<ClientRequest>)>,
}
impl SteamMetadata {
    pub fn new() -> anyhow::Result<Self> {
        let client = steamworks::Client::init_app(480)?;
        client.networking_utils().init_relay_network_access();
        Ok(Self {
            client,
            own_client: None,
        })
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

    async fn create_client(&mut self, _id: u64) -> anyhow::Result<NetworkClient> {
        if let Some((read, write)) = self.own_client.take() {
            Ok(NetworkClient::new(SteamClient::Channel { read, write }))
        } else {
            Ok(NetworkClient::new(SteamClient::Real))
        }
    }

    async fn create_lobby(&mut self) -> anyhow::Result<NetworkHost> {
        let (send, recv) = std::sync::mpsc::channel();
        self.client
            .matchmaking()
            .create_lobby(steamworks::LobbyType::FriendsOnly, 8, move |r| {
                let Err(e) = send.send(r) else { return };
                panic!("{e}");
            });
        let listen_socket = self
            .client
            .networking_sockets()
            .create_listen_socket_p2p(8000, [])?;
        let lobby_id = loop {
            self.client.run_callbacks();
            if let Ok(id) = recv.try_recv() {
                break id?.raw();
            }
        };
        let (kill_send, kill_recv) = oneshot::channel();
        let (msg_send, msg_recv) = mpsc::channel(1000);

        let (mes_send, mes_recv) = mpsc::channel(100);
        let (req_send, req_recv) = mpsc::channel(100);
        self.own_client = Some((mes_recv, req_send));
        std::thread::spawn(move || {
            loop {
                let ev = listen_socket.receive_event();
                match ev {
                    ListenSocketEvent::Connecting(connection_request) => todo!(),
                    ListenSocketEvent::Connected(connected_event) => {
                        todo!()
                    }
                    ListenSocketEvent::Disconnected(disconnected_event) => todo!(),
                }
            }
        });
        Ok(NetworkHost::new(SteamHost {
            lobby_id,
            kill_send,
            msg_recv,
            own_recv: req_recv,
            own_send: mes_send,
        }))
    }
}
