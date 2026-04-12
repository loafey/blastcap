use super::Arg;
use crate::{
    game::state::{EnterDungeonState, Res, State},
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
        addr: u64,
        req: ClientRequest,
        Arg {
            data,
            host,
            last_tick,
        }: Arg<'l>,
    ) -> Res {
        match req {
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
                Ok(Some(EnterDungeonState::new(host.get_clients())))
            }
            ClientRequest::ChangeToEnterDungeon if Some(addr) == data.host_player => {
                host.broadcast(ServerMessage::EnterDungeonState).await?;
                Ok(Some(EnterDungeonState::new(host.get_clients())))
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
