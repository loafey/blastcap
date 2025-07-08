use super::Arg;
use crate::{
    game::{
        actor::{Actor, Controller},
        map::Piece,
        state::{GameStartedState, Res, State},
    },
    network::messages::{ClientRequest, ServerMessage},
};
use math::Vec3;
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

                    let mut gs = GameStartedState::new();
                    let (mut x_list, mut y_list, mut z_list) = (Vec::new(), Vec::new(), Vec::new());
                    for y in 0..16 {
                        for x in 0..(16 - y) {
                            for z in 0..(16 - y) {
                                gs.map.set(Vec3::new(x, y, z), Piece::Ground);
                                x_list.push(x);
                                y_list.push(y);
                                z_list.push(z);
                            }
                        }
                    }

                    arg.host
                        .broadcast(ServerMessage::SpawnMap {
                            x: x_list,
                            y: y_list,
                            z: z_list,
                        })
                        .await?;
                    // let mut posses = [
                    //     Vec2::new(0, 0),
                    //     Vec2::new(0, 15),
                    //     Vec2::new(15, 0),
                    //     Vec2::new(15, 15),
                    // ]
                    // .into_iter()
                    // .cycle();
                    for (id, addr) in self.players.iter().copied().enumerate() {
                        gs.spawn_actor(
                            arg.host,
                            Actor {
                                name: format!("Player {id}"),
                                controller: Controller::Player(addr),
                                position: Vec3::new(15, 1, 15),
                                abilities: Default::default(),
                                health: 10,
                                base_movement: 6,
                                resources: Default::default(),
                            },
                        )
                        .await?;
                    }
                    // let mut i = 0;
                    // loop {
                    //     let position = Vec3::new(
                    //         rand::random_range(0..16),
                    //         rand::random_range(0..16),
                    //         rand::random_range(0..16),
                    //     );

                    //     let mut actor = Actor {
                    //         name: format!("Bot {i}"),
                    //         controller: Controller::Bot,
                    //         position,
                    //         health: 15,
                    //         base_movement: 8,
                    //         abilities: Default::default(),
                    //         resources: Default::default(),
                    //     };
                    //     actor.reset_turn_resources();
                    //     if gs.spawn_actor(arg.host, actor).await? {
                    //         i += 1;
                    //         if i > 64 {
                    //             break;
                    //         }
                    //     }
                    // }
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
