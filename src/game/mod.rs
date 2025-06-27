use tokio::time::Instant;

use crate::{
    game::state::{Arg, LobbyState, State},
    network::NetworkHost,
};
use std::net::SocketAddr;

mod state;

type Map = [[u8; 16]; 16];
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Controller {
    Player(SocketAddr),
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
struct Vec2 {
    x: usize,
    y: usize,
}
impl Vec2 {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}
struct Actor {
    name: String,
    id: usize,
    controller: Controller,
    position: Vec2,
}

#[derive(Default)]
struct ServerData {
    host_player: Option<SocketAddr>,
    tick: usize,
}

pub async fn host_loop(port: u16) -> anyhow::Result<()> {
    let mut host = NetworkHost::tcp(port).await?;

    let mut data = ServerData::default();
    let mut state: Box<dyn State> = LobbyState::new();
    let mut last_tick = Instant::now();
    while let Ok(poll) = host.poll().await {
        if let Some(new_state) = state
            .handle_req(
                poll,
                Arg {
                    data: &mut data,
                    host: &mut host,
                    last_tick: &mut last_tick,
                },
            )
            .await?
        {
            state = new_state;
        }
    }
    Ok(())
}
