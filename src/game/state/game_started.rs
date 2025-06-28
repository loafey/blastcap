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
use std::collections::VecDeque;

type Map = [[Piece; 16]; 16];
#[derive(Default)]
enum Piece {
    #[default]
    Empty,
    Rock,
    Actor(usize),
}

pub struct GameStartedState {
    id_counter: usize,
    actors: Vec<Actor>,
    actor_pointer: usize,
    map: Box<Map>,
    current_turn: Option<Controller>,
    current_id: usize,
}
impl GameStartedState {
    pub async fn next_actor(&mut self, host: &mut NetworkHost) -> anyhow::Result<()> {
        if let Some(actor) = self.actors.get(self.actor_pointer) {
            let addr = match actor.controller {
                Controller::Player(addr) => Some(addr),
            };
            self.current_turn = Some(actor.controller);
            self.current_id = actor.id;
            for cl in host.get_clients() {
                if Some(cl) == addr {
                    host.send(cl, ServerMessage::YourTurn { actor: actor.id })
                        .await?;
                } else {
                    host.send(cl, ServerMessage::ActorTurn { actor: actor.id })
                        .await?;
                }
            }
            self.actor_pointer = (self.actor_pointer + 1) % self.actors.len();
        }
        Ok(())
    }
}
impl GameStartedState {
    pub fn new<I: IntoIterator<Item = Actor>>(actors: I) -> Box<Self> {
        let actors = Vec::from_iter(actors);
        Box::new(Self {
            id_counter: actors.len(),
            actors,
            actor_pointer: 0,
            map: Default::default(),
            current_turn: None,
            current_id: usize::MAX,
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
                println!("UPDATE ON SERVER!!! {x} {y}");
                arg.host
                    .broadcast(ServerMessage::MoveActor {
                        actor: self.current_id,
                        x,
                        y,
                    })
                    .await?;
                self.next_actor(arg.host).await?;
                Ok(None)
            }
            req => self.default_client_request(addr, req, arg).await,
        }
    }
}
