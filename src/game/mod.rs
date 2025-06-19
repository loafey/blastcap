use crate::network::{
    ClientPoll, HostPoll, NetworkClient, NetworkHost,
    messages::{ClientRequest, ServerMessage},
};
use tokio::{
    net::ToSocketAddrs,
    sync::mpsc::{Receiver, Sender},
};

pub async fn client<A: ToSocketAddrs>(
    addr: A,
    server_send: Sender<ServerMessage>,
    mut client_req_recv: Receiver<ClientRequest>,
) -> anyhow::Result<()> {
    let mut client = NetworkClient::tcp(addr).await?;
    let mut tick_counter: usize = 0;
    while let Ok(res) = client.poll().await {
        match res {
            ClientPoll::Message(client_message) => server_send.send(client_message).await?,
            ClientPoll::Tick => {
                tick_counter = tick_counter.wrapping_add(1);
                if let Ok(msg) = client_req_recv.try_recv() {
                    client.send(msg).await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn host(port: u16) -> anyhow::Result<()> {
    let mut host = NetworkHost::tcp(port).await.unwrap();

    let mut tick_counter: usize = 0;
    // let mut tick_time = std::time::Instant::now();
    while let Ok(res) = host.poll().await {
        match res {
            HostPoll::ClientConnected(socket_addr) => {
                println!("SERVER - A user at {socket_addr} connected");
                host.broadcast(ServerMessage::NewUser(socket_addr)).await?;
            }
            HostPoll::ClientRequest { addr, req } => match req {
                ClientRequest::Ping => {
                    let clients = host.get_clients();
                    host.send(addr, ServerMessage::Pong(clients)).await?
                }
                ClientRequest::ChatMessage(msg) => {
                    host.broadcast(ServerMessage::ChatMessage(addr, msg))
                        .await?;
                }
            },
            HostPoll::Tick => tick_counter = tick_counter.wrapping_add(1),
            HostPoll::RemoveClient(socket_addr) => {
                host.remove_client(socket_addr);
                host.broadcast(ServerMessage::UserLeft(socket_addr)).await?;
            }
        }
    }
    Ok(())
}
