use std::{net::SocketAddr, time::Instant};

use crate::network::{
    HostPoll, NetworkHost, TICK_RATE,
    messages::{ClientRequest, ServerMessage},
};

#[derive(Default)]
struct State {
    host_player: Option<SocketAddr>,
    tick: usize,
}

pub async fn host_loop(port: u16) -> anyhow::Result<()> {
    let mut host = NetworkHost::tcp(port).await?;

    let mut state = State::default();
    let mut last_tick = Instant::now();
    while let Ok(res) = host.poll().await {
        match res {
            HostPoll::ClientConnected(socket_addr) => {
                println!("SERVER - A user at {socket_addr} connected");
                host.broadcast(ServerMessage::NewUser(format!("{socket_addr}")))
                    .await?;
                if host.get_client_count() == 1 {
                    state.host_player = Some(socket_addr);
                }
            }
            HostPoll::ClientRequest { addr, req } => match req {
                ClientRequest::Ping => host.send(addr, ServerMessage::Pong).await?,
                ClientRequest::ChatMessage(msg) => {
                    host.broadcast(ServerMessage::ChatMessage(format!("{addr}"), msg))
                        .await?;
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
