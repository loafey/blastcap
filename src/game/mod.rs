use std::{net::SocketAddr, time::Instant};

use crate::network::{
    HostPoll, NetworkHost, TICK_RATE,
    messages::{ClientRequest, ServerMessage},
};

#[derive(Default)]
struct State {
    host_player: Option<SocketAddr>,
    tick: usize,
    started: bool,
}

pub async fn host_loop(port: u16) -> anyhow::Result<()> {
    let mut host = NetworkHost::tcp(port).await?;

    let mut state = State::default();
    let mut last_tick = Instant::now();
    while let Ok(res) = host.poll().await {
        match res {
            HostPoll::ClientConnected(addr) => {
                println!("SERVER - A user at {addr} connected");
                if host.get_client_count() == 1 {
                    state.host_player = Some(addr);
                    host.send(addr, ServerMessage::NotifyHost).await?;
                }
                let raw_clients = host.get_clients();
                let clients = raw_clients
                    .iter()
                    .map(|v| format!("{v}"))
                    .collect::<Vec<_>>();
                for client in raw_clients {
                    if client != addr {
                        host.send(client, ServerMessage::NewUser(format!("{addr}")))
                            .await?;
                    }
                }
                host.send(addr, ServerMessage::PlayerList(clients)).await?;
            }
            HostPoll::ClientRequest { addr, req } => match req {
                ClientRequest::Ping => host.send(addr, ServerMessage::Pong).await?,
                ClientRequest::ChatMessage(msg) => {
                    host.broadcast(ServerMessage::ChatMessage(format!("{addr}"), msg))
                        .await?;
                }
                ClientRequest::RequestMapList => {
                    if Some(addr) != state.host_player {
                        continue;
                    };
                    host.send(
                        addr,
                        ServerMessage::MapList(vec!["SimpleTestMap".to_string()]),
                    )
                    .await?;
                }
                ClientRequest::StartMap(map) => {
                    if Some(addr) != state.host_player {
                        continue;
                    };
                    state.started = true;

                    host.broadcast(ServerMessage::StartMap(map)).await?;
                }
            },
            HostPoll::Tick => {
                state.tick = state.tick.wrapping_add(1);
                const TICK_DELAY: usize = 1;
                if let Some(addr) = state.host_player
                    && state.tick % (TICK_RATE * TICK_DELAY) == 0
                {
                    let msg = ServerMessage::Status {
                        user_count: host.get_client_count(),
                        tick_diff: last_tick.elapsed().as_secs_f32() - const { TICK_DELAY as f32 },
                    };
                    host.send(addr, msg).await?;
                    last_tick = Instant::now()
                }
            }
            HostPoll::RemoveClient(socket_addr) => {
                host.remove_client(socket_addr);
                host.broadcast(ServerMessage::UserLeft(format!("{socket_addr}")))
                    .await?;
            }
        }
    }
    Ok(())
}
