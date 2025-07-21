use crate::{
    game::Arg,
    network::{
        HostPoll, TICK_RATE,
        messages::{ClientRequest, ServerMessage},
    },
};
use std::{any::type_name, time::Instant};

mod game_started;
pub use game_started::GameStartedState;
mod lobby;
pub use lobby::LobbyState;
mod waiting;
pub use waiting::WaitingState;

pub type Res = anyhow::Result<Option<Box<dyn State>>>;

#[async_trait::async_trait]
pub trait State: Sync + Send {
    async fn host_poll_tick<'l>(&mut self, arg: Arg<'l>) -> Res {
        arg.data.tick = arg.data.tick.wrapping_add(1);
        const TICK_DELAY: usize = 1;
        if let Some(addr) = arg.data.host_player
            && arg.data.tick % (TICK_RATE * TICK_DELAY) == 0
        {
            let msg = ServerMessage::Status {
                user_count: arg.host.get_client_count(),
                tick_diff: arg.last_tick.elapsed().as_secs_f32() - const { TICK_DELAY as f32 },
            };
            arg.host.send(addr, msg).await?;
            *arg.last_tick = Instant::now();
        }
        Ok(None)
    }

    async fn host_poll_remove_client<'l>(&mut self, Arg { host, .. }: Arg<'l>, addr: u64) -> Res {
        host.remove_client(addr);
        host.broadcast(ServerMessage::UserLeft(format!("{addr}")))
            .await?;
        Ok(None)
    }

    async fn host_poll_client_connected<'l>(
        &mut self,
        addr: u64,
        Arg { data, host, .. }: Arg<'l>,
    ) -> Res {
        println!("A user at {addr} connected");
        if host.get_client_count() == 1 {
            data.host_player = Some(addr);
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

        Ok(None)
    }

    async fn client_req<'l>(&mut self, addr: u64, req: ClientRequest, arg: Arg<'l>) -> Res {
        self.default_client_request(addr, req, arg).await
    }
    async fn default_client_request<'l>(
        &mut self,
        addr: u64,
        req: ClientRequest,
        arg: Arg<'l>,
    ) -> Res {
        if let ClientRequest::Ping = req {
            arg.host.send(addr, ServerMessage::Pong).await?;
        } else {
            error!(
                "SERVER - please implement \"{req:?}\" for {}",
                type_name::<Self>()
            );
        }
        Ok(None)
    }

    async fn handle_req<'l>(&mut self, poll: HostPoll, arg: Arg<'l>) -> Res {
        match poll {
            HostPoll::ClientConnected(addr) => self.host_poll_client_connected(addr, arg).await,
            HostPoll::ClientRequest { addr, req } => self.client_req(addr, req, arg).await,
            HostPoll::RemoveClient(addr) => self.host_poll_remove_client(arg, addr).await,
            HostPoll::Tick => self.host_poll_tick(arg).await,
            HostPoll::Nothing => Ok(None),
        }
    }
}
