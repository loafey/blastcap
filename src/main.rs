#![feature(impl_trait_in_bindings, never_type)]
mod args;
mod gui;
mod network;

use crate::network::{
    HostPoll, NetworkClient, NetworkHost, TICK_RATE,
    messages::{ClientRequest, ServerMessage},
};
use tokio::net::ToSocketAddrs;

async fn client<A: ToSocketAddrs>(addr: A) -> anyhow::Result<()> {
    let mut client = NetworkClient::tcp(addr).await?;
    let mut tick_counter: usize = 0;
    while let Ok(res) = client.poll().await {
        match res {
            network::ClientPoll::Message(client_message) => {
                println!("CLIENT - Got message {client_message:?}")
            }
            network::ClientPoll::Tick => {
                tick_counter = tick_counter.wrapping_add(1);

                if tick_counter % (TICK_RATE * rand::random_range(2..=7)) == 0 {
                    // println!("CLIENT {i} - sending message!");
                    client
                        .send(ClientRequest::ChatMessage(format!(
                            "I am stinky: {tick_counter}!"
                        )))
                        .await?;
                    if rand::random_range(0..5) == 0 {
                        // println!("CLIENT {i} - I am leaving");
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn host(port: u16) -> anyhow::Result<()> {
    let mut host = NetworkHost::tcp(port).await.unwrap();

    let mut tick_counter: usize = 0;
    let mut tick_time = std::time::Instant::now();
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
                if tick_counter % (TICK_RATE - 1) == 0 {
                    let metrics = tokio::runtime::Handle::current().metrics();
                    println!(
                        "SERVER - time since last check: {:0.03?}s, {} client(s) connected\n\t queue depth: {}, alive tasks: {}, workers: {}",
                        tick_time.elapsed().as_secs_f64(),
                        host.get_client_count(),
                        metrics.global_queue_depth(),
                        metrics.num_alive_tasks(),
                        metrics.num_workers(),
                    );
                    tick_time = std::time::Instant::now();
                }
            }
            HostPoll::RemoveClient(socket_addr) => host.remove_client(socket_addr),
        }
    }
    Ok(())
}

fn main() {
    gui::start();
}
