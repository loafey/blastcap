use super::Arg;
use crate::{
    game::{
        actor::Abilities,
        state::{Res, State, WaitingState},
    },
    network::messages::{ClientRequest, ServerMessage},
};

pub struct LobbyState;
impl LobbyState {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}
#[async_trait::async_trait]
impl State for LobbyState {
    async fn client_req<'l>(
        &mut self,
        addr: std::net::SocketAddr,
        req: ClientRequest,
        Arg {
            data,
            host,
            last_tick,
        }: Arg<'l>,
    ) -> Res {
        match req {
            ClientRequest::ChatMessage(msg) => {
                host.broadcast(ServerMessage::ChatMessage(format!("{addr}"), msg))
                    .await?;
                Ok(None)
            }
            ClientRequest::RequestMapList if Some(addr) == data.host_player => {
                host.send(
                    addr,
                    ServerMessage::MapList(vec!["SimpleTestMap".to_string()]),
                )
                .await?;
                Ok(None)
            }
            ClientRequest::StartMap(map) if Some(addr) == data.host_player => {
                host.broadcast(ServerMessage::StartMap(map)).await?;
                host.broadcast(ServerMessage::AbilityMap(Abilities::get_map().clone()))
                    .await?;
                Ok(Some(WaitingState::new(host.get_clients())))
            }
            req => {
                self.default_client_request(
                    addr,
                    req,
                    Arg {
                        data,
                        host,
                        last_tick,
                    },
                )
                .await
            }
        }
    }
}
