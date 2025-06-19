use crate::network::{
    ClientPoll, ClientRequest, HostPoll, NetworkClientExt, NetworkHostExt, ServerMessage, TICK_RATE,
};
use async_trait::async_trait;
use futures::{StreamExt, stream::FuturesOrdered};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, WriteHalf, split},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc::{Receiver, Sender, channel},
};

pub(super) struct TcpClient {
    write: WriteHalf<TcpStream>,
    recv: Receiver<ServerMessage>,
}
impl TcpClient {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let (send, recv) = channel(1000);
        let (mut read, write) = split(stream);
        let closure: impl Future<Output = anyhow::Result<!>> = async move {
            loop {
                let len = read.read_u32().await? as usize;
                if len > 10000 {
                    eprintln!("large message!");
                    continue;
                }
                let mut buf = vec![0; len];
                let _ = read.read(&mut buf).await?;
                let msg = rkyv::from_bytes::<ServerMessage, rkyv::rancor::Error>(&buf)?;
                send.send(msg).await.expect("client dead!");
            }
        };
        tokio::spawn(async move { closure.await.unwrap() });
        Ok(Self { write, recv })
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
            _ = tokio::time::sleep(std::time::Duration::from_secs_f64(const { 1.0 / TICK_RATE as f64})) => {
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

pub(super) struct TcpHost {
    listener: TcpListener,
    clients: HashMap<SocketAddr, WriteHalf<TcpStream>>,
    recv: Receiver<(SocketAddr, ClientRequest)>,
    send: Sender<(SocketAddr, ClientRequest)>,
}
impl TcpHost {
    pub async fn new(port: u16) -> anyhow::Result<Self> {
        let (send, recv) = channel(1000);
        Ok(Self {
            listener: TcpListener::bind(format!("0.0.0.0:{port}")).await?,
            send,
            recv,
            clients: Default::default(),
        })
    }

    async fn acc(&mut self, (stream, addr): (TcpStream, SocketAddr)) {
        let send = self.send.clone();
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
        tokio::spawn(async move { closure.await.expect("acc crashed") });
    }
}

#[async_trait]
impl NetworkHostExt for TcpHost {
    async fn poll(&mut self) -> anyhow::Result<HostPoll> {
        tokio::select! {
            acc = self.listener.accept() => {
                let (stream, addr) = acc?;
                self.acc((stream, addr)).await;
                Ok(HostPoll::ClientConnected(addr))
            },
            msg = self.recv.recv() => {
                let Some((addr, req)) = msg else { panic!("no clients somehow") };
                Ok(HostPoll::ClientRequest { addr, req })
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs_f64(const { 1.0 / TICK_RATE as f64 })) => {
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

    fn get_clients(&self) -> Vec<SocketAddr> {
        self.clients.keys().copied().collect()
    }
}
