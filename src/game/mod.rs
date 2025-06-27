use crate::{
    game::state::{Arg, LobbyState, State},
    network::NetworkHost,
};
use std::net::SocketAddr;
use tokio::time::Instant;

mod actor;
mod state;

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
