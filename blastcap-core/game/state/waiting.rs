use super::Arg;
use crate::{
    game::{
        actor::{Actor, Controller},
        state::{GameStartedState, Res, State},
    },
    network::messages::ClientRequest,
};
use math::Vec2;
use std::{collections::HashSet, net::SocketAddr};

pub struct WaitingState {
    waiting_for: HashSet<SocketAddr>,
    players: HashSet<SocketAddr>,
}
impl WaitingState {
    pub fn new<I: IntoIterator<Item = SocketAddr>>(waiting_for: I) -> Box<Self> {
        Box::new(Self {
            waiting_for: HashSet::from_iter(waiting_for),
            players: Default::default(),
        })
    }
}
#[async_trait::async_trait]
impl State for WaitingState {
    async fn client_req<'l>(&mut self, addr: SocketAddr, req: ClientRequest, arg: Arg<'l>) -> Res {
        match req {
            ClientRequest::NotifyReady => {
                if self.waiting_for.remove(&addr) {
                    self.players.insert(addr);
                }
                //
                if self.waiting_for.is_empty() {
                    println!(
                        "SERVER - Starting game with player actor controllers: {:?}",
                        self.players
                    );
                    //
                    let mut posses = [
                        Vec2::new(0, 0),
                        Vec2::new(0, 15),
                        Vec2::new(15, 0),
                        Vec2::new(15, 15),
                    ]
                    .into_iter()
                    .cycle();

                    let mut gs = GameStartedState::new();
                    for (id, addr) in self.players.iter().copied().enumerate() {
                        gs.spawn_actor(
                            arg.host,
                            Actor {
                                name: format!("Player {id}"),
                                controller: Controller::Player(addr),
                                position: posses.next().unwrap(),
                                abilities: Default::default(),
                                health: 10,
                                base_movement: 6,
                            },
                        )
                        .await?;
                    }
                    for i in 0..1 {
                        gs.spawn_actor(
                            arg.host,
                            Actor {
                                name: format!("Bot {i}"),
                                controller: Controller::Bot,
                                position: Vec2::new(
                                    rand::random_range(0..16),
                                    rand::random_range(0..16),
                                ),
                                abilities: Default::default(),
                                health: 15,
                                base_movement: 8,
                            },
                        )
                        .await?;
                    }
                    gs.next_actor(arg.host).await?;
                    Ok(Some(gs))
                } else {
                    Ok(None)
                }
            }
            req => self.default_client_request(addr, req, arg).await,
        }
    }
}
