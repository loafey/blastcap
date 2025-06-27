use std::{any::type_name, net::SocketAddr};

use tokio::time::Instant;

use crate::{
    game::{Controller, ServerData},
    network::{
        HostPoll, NetworkHost, TICK_RATE,
        messages::{ClientRequest, ServerMessage},
    },
};
pub struct Arg<'l> {
    pub data: &'l mut ServerData,
    pub host: &'l mut NetworkHost,
    pub last_tick: &'l mut Instant,
}

type Res = anyhow::Result<Option<Box<dyn State>>>;

#[async_trait::async_trait]
pub trait State {
    async fn host_poll_tick<'l>(
        &mut self,
        Arg {
            data,
            host,
            last_tick,
        }: Arg<'l>,
    ) -> Res {
        data.tick = data.tick.wrapping_add(1);
        const TICK_DELAY: usize = 1;
        if let Some(addr) = data.host_player
            && data.tick % (TICK_RATE * TICK_DELAY) == 0
        {
            let msg = ServerMessage::Status {
                user_count: host.get_client_count(),
                tick_diff: last_tick.elapsed().as_secs_f32() - const { TICK_DELAY as f32 },
            };
            host.send(addr, msg).await?;
            *last_tick = Instant::now();
        }
        Ok(None)
    }

    async fn host_poll_remove_client<'l>(
        &mut self,
        Arg {
            data,
            host,
            last_tick,
        }: Arg<'l>,
        addr: SocketAddr,
    ) -> Res {
        host.remove_client(addr);
        host.broadcast(ServerMessage::UserLeft(format!("{addr}")))
            .await?;
        Ok(None)
    }

    async fn host_poll_client_connected<'l>(
        &mut self,
        addr: SocketAddr,
        Arg { data, host, .. }: Arg<'l>,
    ) -> Res {
        println!("SERVER - A user at {addr} connected");
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

    async fn host_poll_client_request<'l>(
        &mut self,
        addr: SocketAddr,
        req: ClientRequest,
        arg: Arg<'l>,
    ) -> Res {
        match req {
            ClientRequest::Ping => host.send(addr, ServerMessage::Pong).await?,
            ClientRequest::ChatMessage(msg) => {
                self.client_request_chat_message(addr, msg, arg).await
            }
            ClientRequest::RequestMapList => self.client_request_map_list(addr, arg).await,
            ClientRequest::StartMap(map) => self.client_request_start_map(addr, map, arg).await,
            ClientRequest::NotifyReady => self.client_request_notify_ready(addr, arg).await,
            ClientRequest::MoveActor(x, y) => self.client_request_move_actor(addr, x, y, arg).await,
        }
    }

    async fn client_request_chat_message<'l>(
        &mut self,
        _addr: SocketAddr,
        _msg: String,
        _arg: Arg<'l>,
    ) -> Res {
        eprintln!(
            "SERVER - please implement \"client_request_chat_message\" for {}",
            type_name::<Self>()
        );
        Ok(None)
    }

    async fn client_request_map_list<'l>(&mut self, _addr: SocketAddr, _arg: Arg<'l>) -> Res {
        eprintln!(
            "SERVER - please implement \"client_request_map_list\" for {}",
            type_name::<Self>()
        );
        Ok(None)
    }

    async fn client_request_start_map<'l>(
        &mut self,
        _addr: SocketAddr,
        _map: String,
        _arg: Arg<'l>,
    ) -> Res {
        eprintln!(
            "SERVER - please implement \"client_request_start_map\" for {}",
            type_name::<Self>()
        );
        Ok(None)
    }

    async fn client_request_notify_ready<'l>(&mut self, _addr: SocketAddr, _arg: Arg<'l>) -> Res {
        eprintln!(
            "SERVER - please implement \"client_request_notify_ready\" for {}",
            type_name::<Self>()
        );
        Ok(None)
    }

    async fn client_request_move_actor<'l>(
        &mut self,
        _addr: SocketAddr,
        _x: usize,
        _y: usize,
        _arg: Arg<'l>,
    ) -> Res {
        eprintln!(
            "SERVER - please implement \"client_request_move_actor\" for {}",
            type_name::<Self>()
        );
        Ok(None)
    }

    async fn handle_req<'l>(&mut self, poll: HostPoll, arg: Arg<'l>) -> Res {
        match poll {
            HostPoll::ClientConnected(addr) => self.host_poll_client_connected(addr, arg).await,
            HostPoll::ClientRequest { addr, req } => {
                self.host_poll_client_request(addr, req, arg).await
            }
            HostPoll::RemoveClient(addr) => self.host_poll_remove_client(arg, addr).await,
            HostPoll::Tick => self.host_poll_tick(arg).await,
        }
    }
}
