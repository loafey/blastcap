use crate::network::{
    BOT_ADDR, ClientPoll, HostPoll, MetadataExt, NetworkClient, NetworkClientExt, NetworkHost,
    NetworkHostExt, TICK_RATE,
    channel::{Channel, DisjointChannel, disjoint},
    messages::{ClientRequest, ServerMessage},
    tick,
};
use anyhow::Context;
use async_trait::async_trait;
use futures_concurrency::future::Join;
use select::Select;
use smol::{channel, stream::StreamExt};
use std::{collections::HashMap, time::Duration};
use steamworks::{
    Client, LobbyEnter, SteamId,
    networking_types::{ListenSocketEvent, SendFlags},
};

pub enum SteamClient {
    Real,
    Channel(DisjointChannel<ClientRequest, ServerMessage>),
}
#[async_trait]
impl NetworkClientExt for SteamClient {
    async fn poll(&mut self) -> anyhow::Result<ClientPoll> {
        let fut = match self {
            SteamClient::Real => todo!("steam real poll"),
            SteamClient::Channel(dis) => dis.recv(),
        };
        select::select!(
            (fut, |msg| {
                let Some(msg) = msg else {
                    panic!("no clients somehow")
                };
                Ok(ClientPoll::Message(msg))
            }),
            (tick(), |_| { Ok(ClientPoll::Tick) })
        )
    }
    async fn send(&mut self, req: ClientRequest) -> anyhow::Result<()> {
        match self {
            SteamClient::Real => todo!("steam real send"),
            SteamClient::Channel(dis) => dis.send(req).await?,
        };
        Ok(())
    }
}
struct SteamHost<'a> {
    _lobby_id: u64,
    first_poll: bool,
    host_id: u64,
    clients: HashMap<u64, channel::Sender<ServerMessage>>,
    client_req: Channel<(u64, ClientRequest)>,
    listener: Channel<ListenSocketEvent>,
    own: DisjointChannel<ServerMessage, ClientRequest>,
    mock: Channel<ClientRequest>,
    poll: Select<'a, HostPoll>,
}
impl<'a> SteamHost<'a> {
    async fn handle_listen(&mut self, ev: ListenSocketEvent) -> anyhow::Result<HostPoll> {
        match ev {
            ListenSocketEvent::Connecting(req) => {
                req.accept()?;
                Ok(HostPoll::Nothing)
            }
            ListenSocketEvent::Connected(conn) => {
                let id = conn
                    .remote()
                    .steam_id()
                    .with_context(|| "missing Steam id")?
                    .raw();
                let (client_send, mut client_recv) = channel::unbounded();
                let req_send = self.client_req.sender();
                self.clients.insert(id, client_send);
                let mut conn = conn.take_connection();
                smol::unblock(move || {
                    fn tick() {
                        std::thread::sleep(Duration::from_secs_f64(
                            const { 1.0 / TICK_RATE as f64 },
                        ));
                    }
                    loop {
                        let messages = match conn.receive_messages(100) {
                            Ok(o) => o,
                            Err(e) => {
                                error!("error getting client({id}) messages: {e}");
                                tick();
                                continue;
                            }
                        };
                        let mut sleep = messages.is_empty();
                        for message in messages {
                            let message = match rkyv::from_bytes::<ClientRequest, rkyv::rancor::Error>(
                                message.data(),
                            ) {
                                Ok(o) => o,
                                Err(e) => {
                                    error!("error deserializing client request: {e}");
                                    tick();
                                    continue;
                                }
                            };
                            req_send.send_blocking((id, message)).unwrap();
                        }
                        if let Ok(msg) = client_recv.try_recv() {
                            sleep = false;
                            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&msg).unwrap();
                            if let Err(e) = conn.send_message(&bytes, SendFlags::RELIABLE) {
                                error!("failed sending client({id}) message: {e}");
                                break;
                            }
                        }
                        if sleep {
                            tick()
                        }
                    }
                });
                Ok(HostPoll::ClientConnected(id))
            }
            ListenSocketEvent::Disconnected(ev) => {
                let id = ev
                    .remote()
                    .steam_id()
                    .with_context(|| "missing Steam id")?
                    .raw();
                self.clients.remove(&id);
                Ok(HostPoll::RemoveClient(id))
            }
        }
    }
}

#[async_trait]
impl<'a> NetworkHostExt for SteamHost<'a> {
    async fn mock(&mut self, req: ClientRequest) -> anyhow::Result<()> {
        self.mock.send(req).await?;
        Ok(())
    }
    async fn poll(&mut self) -> anyhow::Result<HostPoll> {
        // self.listen_socket.events();
        if self.first_poll {
            self.first_poll = false;
            return Ok(HostPoll::ClientConnected(self.host_id));
        }
        select::select!(
            (self.listener.recv(), |ev| {
                let Some(ev) = ev else { unreachable!() };
                self.handle_listen(ev).await
            }),
            (self.own.recv(), |own| {
                let Some(req) = own else { unreachable!() };
                Ok(HostPoll::ClientRequest {
                    addr: self.host_id,
                    req,
                })
            }),
            (self.mock.recv(), |mocked| {
                let Some(req) = mocked else { unreachable!() };
                Ok(HostPoll::ClientRequest {
                    addr: BOT_ADDR,
                    req,
                })
            }),
            (tick(), |_| Ok(HostPoll::Tick))
        )
    }

    async fn send(&mut self, addr: u64, req: ServerMessage) -> anyhow::Result<()> {
        if addr == self.host_id {
            self.own.send(req).await?;
        } else {
            let Some(cli) = self.clients.get(&addr) else {
                return Ok(());
            };
            cli.send(req).await?;
        }
        Ok(())
    }

    async fn broadcast(&mut self, req: ServerMessage) -> anyhow::Result<()> {
        self.own.send(req.clone()).await?;
        let mut tasks = Vec::new();
        for writer in self.clients.values_mut() {
            tasks.push(async { _ = writer.send(req.clone()).await });
        }
        tasks.join();

        Ok(())
    }

    fn remove_client(&mut self, _addr: u64) {
        todo!("remove_client")
    }

    fn get_clients(&self) -> Vec<u64> {
        let mut clients = self.clients.keys().copied().collect::<Vec<_>>();
        clients.push(self.host_id);
        clients
    }

    fn get_client_count(&self) -> u32 {
        (self.clients.len() + 1) as u32
    }
}

pub struct SteamMetadata {
    client: Client,
    own_client: Option<DisjointChannel<ClientRequest, ServerMessage>>,
}
impl SteamMetadata {
    pub fn new() -> anyhow::Result<Self> {
        let client = steamworks::Client::init_app(480)?;
        client.networking_utils().init_relay_network_access();
        client.register_callback(|l: LobbyEnter| trace!("lobby enter: {l:?}"));
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
        if let Some(dis) = self.own_client.take() {
            Ok(NetworkClient::new(SteamClient::Channel(dis)))
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

        let (own_client, own) = disjoint();
        self.own_client = Some(own_client);

        let listener = Channel::new();
        let listener_send = listener.sender();
        std::thread::spawn(move || {
            loop {
                let ev = listen_socket.receive_event();
                listener_send.send_blocking(ev).unwrap();
            }
        });
        Ok(NetworkHost::new(SteamHost {
            _lobby_id: lobby_id,
            own,
            first_poll: true,
            mock: Channel::new(),
            host_id: self.get_my_id(),
            clients: HashMap::new(),
            client_req: Channel::new(),
            listener,
            poll: select::repeat!(),
        }))
    }
}
