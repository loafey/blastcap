#![feature(impl_trait_in_bindings, never_type)]
mod args;
mod network;

use crate::network::{
    HostPoll, NetworkClient, NetworkHost, TICK_RATE,
    messages::{ClientRequest, ServerMessage},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    for i in 0..2 {
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs_f64(0.5)).await;
            let mut client = NetworkClient::tcp("localhost:4000").await.unwrap();
            let mut tick_counter: usize = 0;
            while let Ok(res) = client.poll().await {
                match res {
                    network::ClientPoll::Message(client_message) => {
                        println!("CLIENT {i} - Got message {client_message:?}")
                    }
                    network::ClientPoll::Tick => {
                        tick_counter = tick_counter.wrapping_add(1);

                        if tick_counter % (TICK_RATE * rand::random_range(2..=7)) == 0 {
                            // println!("CLIENT {i} - sending message!");
                            client
                                .send(ClientRequest::ChatMessage(format!(
                                    "I am stinky {i}: {tick_counter}!"
                                )))
                                .await
                                .unwrap();
                        }
                    }
                }
            }
        });
    }

    let mut host = NetworkHost::tcp(4000).await.unwrap();

    let mut tick_counter: usize = 0;
    while let Ok(res) = host.poll().await {
        match res {
            HostPoll::ClientConnected(socket_addr) => {
                println!("SERVER - A user at {socket_addr} connected")
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
            HostPoll::Tick => {
                tick_counter = tick_counter.wrapping_add(1);
            }
        }
    }
    println!("Server down!");
    Ok(())
}
