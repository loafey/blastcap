use math::Vec2;

use super::Arg;
use crate::{
    game::{
        actor::{Actor, Controller},
        state::{Res, State},
    },
    network::{
        NetworkHost,
        messages::{ClientRequest, ServerMessage},
    },
};
use std::{collections::VecDeque, mem::swap};

type Map = [[Piece; 16]; 16];
#[derive(Default)]
enum Piece {
    #[default]
    Empty,
    Rock,
    Actor(usize),
}
impl std::fmt::Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "."),
            Self::Rock => write!(f, "#"),
            Self::Actor(arg0) => write!(f, "{arg0}"),
        }
    }
}

pub struct GameStartedState {
    actors: Vec<Actor>,
    actor_pointer: usize,
    map: Box<Map>,
    current_turn: Option<Controller>,
}
impl GameStartedState {
    pub async fn spawn_actor(
        &mut self,
        host: &mut NetworkHost,
        actor: Actor,
    ) -> anyhow::Result<()> {
        let Vec2 { x, y } = actor.position;
        let Some(Piece::Empty) = self.map.get(y).and_then(|r| r.get(x)) else {
            return Ok(());
        };

        let id = self.actors.len();
        self.map[y][x] = Piece::Actor(id);
        host.broadcast(ServerMessage::SpawnPlayer {
            name: actor.name.clone(),
            id,
            x,
            y,
        })
        .await?;
        self.actors.push(actor);

        Ok(())
    }

    pub async fn next_actor(&mut self, host: &mut NetworkHost) -> anyhow::Result<()> {
        if let Some(actor) = self.actors.get(self.actor_pointer) {
            let addr = match actor.controller {
                Controller::Player(addr) => Some(addr),
            };
            self.current_turn = Some(actor.controller);
            for cl in host.get_clients() {
                if Some(cl) == addr {
                    host.send(
                        cl,
                        ServerMessage::YourTurn {
                            actor: self.actor_pointer,
                        },
                    )
                    .await?;
                } else {
                    host.send(
                        cl,
                        ServerMessage::ActorTurn {
                            actor: self.actor_pointer,
                        },
                    )
                    .await?;
                }
            }
            self.actor_pointer = (self.actor_pointer + 1) % self.actors.len();
        }
        Ok(())
    }

    async fn pathfind(&self, from: Vec2, to: Vec2) -> Option<(Vec<Vec2>, usize)> {
        pathfinding::directed::astar::astar(
            &from,
            |Vec2 { x, y }| {
                let (nx, ny) = (*x as isize, *y as isize);
                let mut neighs = Vec::with_capacity(9);
                for y in -1..=1 {
                    for x in -1..=1 {
                        let y = (ny + y) as usize;
                        let x = (nx + x) as usize;
                        if let Some(Piece::Empty) = self.map.get(y).and_then(|r| r.get(x)) {
                            neighs.push((Vec2::new(x, y), 1));
                        }
                    }
                }
                neighs
            },
            |pos| pos.distance(to),
            |a| *a == to,
        )
    }
}
impl GameStartedState {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            actors: Vec::new(),
            actor_pointer: 0,
            map: Default::default(),
            current_turn: None,
        })
    }
}
#[async_trait::async_trait]
impl State for GameStartedState {
    async fn client_req<'l>(
        &mut self,
        addr: std::net::SocketAddr,
        req: ClientRequest,
        arg: Arg<'l>,
    ) -> Res {
        match req {
            ClientRequest::ChatMessage(msg) => {
                arg.host
                    .broadcast(ServerMessage::ChatMessage(format!("{addr}"), msg))
                    .await?;
                Ok(None)
            }
            ClientRequest::MoveActor(x, y)
                if Some(Controller::Player(addr)) == self.current_turn =>
            {
                let Some(Piece::Empty) = self.map.get(y).and_then(|r| r.get(x)) else {
                    return Ok(None);
                };
                let Vec2 { x: old_x, y: old_y } = self.actors[self.actor_pointer].position;
                let path = self
                    .pathfind(Vec2::new(old_x, old_y), Vec2::new(x, y))
                    .await;
                let Some((path, _)) = path else {
                    return Ok(None);
                };
                swap(
                    unsafe {
                        std::mem::transmute::<&mut Piece, &'static mut Piece>(&mut self.map[y][x])
                    },
                    &mut self.map[old_y][old_x],
                );
                self.actors[self.actor_pointer].position = Vec2::new(x, y);
                let mut x_list = Vec::new();
                let mut y_list = Vec::new();
                for Vec2 { x, y } in path {
                    x_list.push(x);
                    y_list.push(y);
                }
                arg.host
                    .broadcast(ServerMessage::MoveActor {
                        actor: self.actor_pointer,
                        x: x_list,
                        y: y_list,
                    })
                    .await?;
                self.next_actor(arg.host).await?;
                Ok(None)
            }
            req => self.default_client_request(addr, req, arg).await,
        }
    }
}
