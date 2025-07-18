use crate::network::{
    BOT_ADDR, ClientPoll, ClientRequest, HOST_ADDR, HostPoll, IdentityExt, MetadataExt,
    NetworkClient, NetworkClientExt, NetworkHost, NetworkHostExt, ServerMessage, SocketAddrExt,
    channel::{Channel, DisjointChannel, disjoint},
    tick,
};
use async_trait::async_trait;
use futures_concurrency::future::Join;
use smol::{
    channel,
    io::{AsyncReadExt, AsyncWriteExt, WriteHalf, split},
    net::{AsyncToSocketAddrs, TcpListener, TcpStream},
};
use std::{collections::HashMap, net::SocketAddr};

enum TcpClient {
    Real {
        write: WriteHalf<TcpStream>,
        recv: channel::Receiver<ServerMessage>,
    },
    Channel(DisjointChannel<ClientRequest, ServerMessage>),
}

impl TcpClient {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new<A: AsyncToSocketAddrs>(addr: A) -> anyhow::Result<TcpClient> {
        let stream = TcpStream::connect(addr).await?;
        let (send, recv) = channel::unbounded();
        let (mut read, write) = split(stream);
        let closure: impl Future<Output = anyhow::Result<()>> = async move {
            loop {
                let mut len = [0u8; 4];
                read.read_exact(&mut len).await?;
                let len = u32::from_ne_bytes(len) as usize;
                let mut buf = vec![0; len];
                let _ = read.read(&mut buf).await?;
                let msg = rkyv::from_bytes::<ServerMessage, rkyv::rancor::Error>(&buf)?;
                if let Err(e) = send.send(msg).await {
                    error!("CLIENT - error getting message: {e}");
                    break;
                };
            }
            Ok(())
        };
        smol::spawn(async move { closure.await.unwrap() });
        Ok(Self::Real { write, recv })
    }
}

#[async_trait]
impl NetworkClientExt for TcpClient {
    async fn poll(&mut self) -> anyhow::Result<ClientPoll> {
        let fut = async {
            match self {
                TcpClient::Real { recv, .. } => recv.recv().await.ok(),
                TcpClient::Channel(dis) => dis.recv().await,
            }
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
            TcpClient::Real { write, .. } => {
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&req)?;
                write.write_all(&(bytes.len() as u32).to_ne_bytes()).await?;
                write.write_all(&bytes).await?;
            }
            TcpClient::Channel(dis) => dis.send(req).await?,
        }
        Ok(())
    }
}

struct TcpHost {
    first_poll: bool,
    listener: TcpListener,
    clients: HashMap<SocketAddr, WriteHalf<TcpStream>>,
    own: DisjointChannel<ServerMessage, ClientRequest>,
    client_req: Channel<(SocketAddr, ClientRequest)>,
    kill: Channel<SocketAddr>,
    mock: Channel<ClientRequest>,
}
impl TcpHost {
    pub async fn new(
        port: u16,
    ) -> anyhow::Result<(Self, DisjointChannel<ClientRequest, ServerMessage>)> {
        let (a, b) = disjoint();
        Ok((
            Self {
                first_poll: true,
                listener: TcpListener::bind(format!("0.0.0.0:{port}")).await?,
                client_req: Channel::new(),
                kill: Channel::new(),
                clients: Default::default(),
                mock: Channel::new(),
                own: a,
            },
            b,
        ))
    }

    async fn acc(&mut self, (stream, addr): (TcpStream, SocketAddr)) {
        let send = self.client_req.sender();
        let kill_send = self.kill.sender();
        let (mut read, write) = split(stream);
        self.clients.insert(addr, write);
        let closure: impl Future<Output = anyhow::Result<!>> = async move {
            loop {
                let mut len = [0u8; 4];
                read.read_exact(&mut len).await?;
                let len = u32::from_ne_bytes(len) as usize;
                if len > 10000 {
                    warn!("large message from {addr}!");
                    continue;
                }
                let mut buf = vec![0; len];
                let _ = read.read(&mut buf).await?;
                let msg = rkyv::from_bytes::<ClientRequest, rkyv::rancor::Error>(&buf)?;
                send.send((addr, msg)).await.expect("server has died!");
            }
        };
        smol::spawn(async move {
            let Err(e) = closure.await;
            warn!("SERVER - recv loop for {addr} crashed: {e}");
            kill_send.send(addr).await.expect("server is dead");
        });
    }
}

#[async_trait]
impl NetworkHostExt for TcpHost {
    async fn mock(&mut self, req: ClientRequest) -> anyhow::Result<()> {
        self.mock.send(req).await?;
        Ok(())
    }
    async fn poll(&mut self) -> anyhow::Result<HostPoll> {
        if self.first_poll {
            self.first_poll = false;
            return Ok(HostPoll::ClientConnected(HOST_ADDR));
        }
        select::select!(
            (self.listener.accept(), |acc| {
                let (stream, addr) = acc?;
                self.acc((stream, addr)).await;
                Ok(HostPoll::ClientConnected(addr.raw()))
            }),
            (self.kill.recv(), |remove| {
                let Some(addr) = remove else { unreachable!() };
                Ok(HostPoll::RemoveClient(addr.raw()))
            }),
            (self.own.recv(), |own| {
                let Some(req) = own else { unreachable!() };
                Ok(HostPoll::ClientRequest {
                    addr: HOST_ADDR,
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
            (self.client_req.recv(), |msg| {
                let Some((addr, req)) = msg else {
                    unreachable!()
                };
                Ok(HostPoll::ClientRequest {
                    addr: addr.raw(),
                    req,
                })
            }),
            (tick(), |_| { Ok(HostPoll::Tick) })
        )
    }

    async fn send(&mut self, addr: u64, req: ServerMessage) -> anyhow::Result<()> {
        let addr = SocketAddr::from_raw(addr);
        if addr.raw().is_host() {
            self.own.send(req).await?;
        } else {
            let Some(writer) = self.clients.get_mut(&addr) else {
                let msg = format!("SERVER - client {addr} does not exist");
                return Err(anyhow::Error::msg(msg));
            };
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&req)?;
            writer
                .write_all(&(bytes.len() as u32).to_ne_bytes())
                .await?;
            writer.write_all(&bytes).await?;
        }
        Ok(())
    }

    async fn broadcast(&mut self, req: ServerMessage) -> anyhow::Result<()> {
        _ = self.own.send(req.clone()).await;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&req)?;
        let mut tasks = Vec::new();
        for writer in self.clients.values_mut() {
            tasks.push(async {
                let _ = writer.write(&(bytes.len() as u32).to_ne_bytes()).await;
                let _ = writer.write_all(&bytes).await;
            });
        }
        tasks.join().await;
        Ok(())
    }

    fn remove_client(&mut self, addr: u64) {
        self.clients.remove(&SocketAddr::from_raw(addr));
    }

    fn get_clients(&self) -> Vec<u64> {
        let mut clients = self
            .clients
            .keys()
            .copied()
            .map(|s| s.raw())
            .collect::<Vec<_>>();
        clients.push(HOST_ADDR);
        clients
    }

    fn get_client_count(&self) -> u32 {
        self.clients.len() as u32 + 1
    }
}

pub struct TcpMetadata {
    _id: u64,
    host_channel: Option<DisjointChannel<ClientRequest, ServerMessage>>,
}
impl TcpMetadata {
    pub fn new() -> Self {
        Self {
            host_channel: None,
            _id: rand::random(),
        }
    }
}
#[async_trait]
impl MetadataExt for TcpMetadata {
    fn get_avatar(&self, _id: u64) -> Option<(Vec<u8>, u16, u16)> {
        None
    }

    fn get_my_id(&self) -> u64 {
        self._id
    }

    fn get_name(&self, id: u64) -> anyhow::Result<String> {
        Ok(format!("{id}"))
    }

    async fn tick(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn create_client(&mut self, _lobby: u64) -> anyhow::Result<NetworkClient> {
        let client = match self.host_channel.take() {
            Some(dis) => TcpClient::Channel(dis),
            _ => TcpClient::new("0.0.0.0:8000").await?,
        };
        Ok(NetworkClient::new(client))
    }

    async fn create_lobby(&mut self) -> anyhow::Result<NetworkHost> {
        let (host, dis) = TcpHost::new(8000).await?;
        self.host_channel = Some(dis);
        Ok(NetworkHost::new(host))
    }
}
