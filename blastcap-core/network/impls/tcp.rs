use crate::network::{
    ClientPoll, ClientRequest, HostPoll, LOCAL_ADDR, MetadataExt, NetworkClientExt, NetworkHostExt,
    ServerMessage, channel::Channel, tick,
};
use async_trait::async_trait;
use futures::{StreamExt, stream::FuturesOrdered};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, WriteHalf, split},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc::{Receiver, Sender, channel},
};

pub struct TcpClient {
    write: WriteHalf<TcpStream>,
    recv: Receiver<ServerMessage>,
}
impl TcpClient {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new<A: ToSocketAddrs>(addr: A) -> anyhow::Result<Box<dyn NetworkClientExt>> {
        let stream = TcpStream::connect(addr).await?;
        let (send, recv) = channel(1000);
        let (mut read, write) = split(stream);
        let closure: impl Future<Output = anyhow::Result<()>> = async move {
            loop {
                let len = read.read_u32().await? as usize;
                let mut buf = vec![0; len];
                let _ = read.read(&mut buf).await?;
                let msg = rkyv::from_bytes::<ServerMessage, rkyv::rancor::Error>(&buf)?;
                if let Err(e) = send.send(msg).await {
                    eprintln!("CLIENT - error getting message: {e}");
                    break;
                };
            }
            Ok(())
        };
        tokio::spawn(async move { closure.await.unwrap() });
        Ok(Box::new(Self { write, recv }))
    }
}

#[async_trait]
impl NetworkClientExt for TcpClient {
    async fn poll(&mut self) -> anyhow::Result<ClientPoll> {
        tokio::select! {
            msg = self.recv.recv() => {
                let Some(msg) = msg else { panic!("no clients somehow") };
                Ok(ClientPoll::Message(msg))
            }
            _ = tick() => {
                Ok(ClientPoll::Tick)
            }
        }
    }

    async fn send(&mut self, req: ClientRequest) -> anyhow::Result<()> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&req)?;
        self.write.write_u32(bytes.len() as u32).await?;
        self.write.write_all(&bytes).await?;
        Ok(())
    }
}

pub struct TcpHost {
    listener: TcpListener,
    clients: HashMap<SocketAddr, WriteHalf<TcpStream>>,
    recv: Receiver<(SocketAddr, ClientRequest)>,
    send: Sender<(SocketAddr, ClientRequest)>,
    kill_recv: Receiver<SocketAddr>,
    kill_send: Sender<SocketAddr>,
    mock: Channel<ClientRequest>,
}
impl TcpHost {
    pub async fn new(port: u16) -> anyhow::Result<Self> {
        let (send, recv) = channel(1000);
        let (kill_send, kill_recv) = channel(10);
        Ok(Self {
            listener: TcpListener::bind(format!("0.0.0.0:{port}")).await?,
            send,
            recv,
            kill_send,
            kill_recv,
            clients: Default::default(),
            mock: Channel::new(10),
        })
    }

    async fn acc(&mut self, (stream, addr): (TcpStream, SocketAddr)) {
        let send = self.send.clone();
        let kill_send = self.kill_send.clone();
        let (mut read, write) = split(stream);
        self.clients.insert(addr, write);
        let closure: impl Future<Output = anyhow::Result<!>> = async move {
            loop {
                let len = read.read_u32().await? as usize;
                if len > 10000 {
                    eprintln!("large message from {addr}!");
                    continue;
                }
                let mut buf = vec![0; len];
                let _ = read.read(&mut buf).await?;
                let msg = rkyv::from_bytes::<ClientRequest, rkyv::rancor::Error>(&buf)?;
                send.send((addr, msg)).await.expect("server has died!");
            }
        };
        tokio::spawn(async move {
            let Err(e) = closure.await;
            eprintln!("SERVER - recv loop for {addr} crashed: {e}");
            kill_send.send(addr).await.expect("server is dead");
        });
    }
}

#[async_trait]
impl NetworkHostExt for TcpHost {
    async fn mock(&mut self, req: ClientRequest) -> anyhow::Result<()> {
        self.mock.send.send(req).await?;
        Ok(())
    }
    async fn poll(&mut self) -> anyhow::Result<HostPoll> {
        tokio::select! {
            acc = self.listener.accept() => {
                let (stream, addr) = acc?;
                self.acc((stream, addr)).await;
                Ok(HostPoll::ClientConnected(addr))
            },
            remove = self.kill_recv.recv() => {
                let Some(addr) = remove else { unreachable!() };
                Ok(HostPoll::RemoveClient(addr))
            },
            mocked = self.mock.recv.recv() => {
                let Some(req) = mocked else { unreachable!() };
                Ok(HostPoll::ClientRequest { addr: *LOCAL_ADDR, req })
            }
            msg = self.recv.recv() => {
                let Some((addr, req)) = msg else { unreachable!() };
                Ok(HostPoll::ClientRequest { addr, req })
            }
            _ = tick() => {
                Ok(HostPoll::Tick)
            }
        }
    }

    async fn send(&mut self, addr: SocketAddr, req: ServerMessage) -> anyhow::Result<()> {
        let Some(writer) = self.clients.get_mut(&addr) else {
            let msg = format!("SERVER - client {addr} does not exist");
            return Err(anyhow::Error::msg(msg));
        };
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&req)?;
        writer.write_u32(bytes.len() as u32).await?;
        writer.write_all(&bytes).await?;
        Ok(())
    }

    async fn broadcast(&mut self, req: ServerMessage) -> anyhow::Result<()> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&req)?;
        let mut tasks = FuturesOrdered::new();
        for writer in self.clients.values_mut() {
            tasks.push_back(async {
                let _ = writer.write_u32(bytes.len() as u32).await;
                let _ = writer.write_all(&bytes).await;
            });
        }
        while (tasks.next().await).is_some() {}
        Ok(())
    }

    fn remove_client(&mut self, addr: SocketAddr) {
        self.clients.remove(&addr);
    }

    fn get_clients(&self) -> Vec<SocketAddr> {
        self.clients.keys().copied().collect()
    }

    fn get_client_count(&self) -> u32 {
        self.clients.len() as u32
    }
}

pub struct TcpMetadata {
    _id: u64,
}
impl TcpMetadata {
    pub fn new() -> Self {
        Self {
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

    fn create_lobby(&self) -> anyhow::Result<u64> {
        Ok(0)
    }
}
