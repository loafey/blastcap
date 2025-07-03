use super::Arg;
use crate::{
    constants::TILES_PER_SECOND,
    game::{
        actor::{Actor, Controller},
        state::{Res, State},
    },
    network::{
        NetworkHost,
        channel::Channel,
        messages::{ClientRequest, ServerMessage},
    },
};
use math::Vec2;
use std::{mem::swap, pin::Pin, time::Duration};

type Map = [[Piece; 16]; 16];
#[derive(Default)]
enum Piece {
    #[default]
    Empty,
    Actor(usize),
}

type Callback = Box<
    dyn FnOnce(
            &'static mut GameStartedState,
            Arg<'static>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send,
>;
pub struct GameStartedState {
    actors: Vec<Actor>,
    actor_pointer: usize,
    map: Box<Map>,
    current_turn: Option<Controller>,
    waiting: bool,
    callbacks: Channel<Callback>,
}
impl GameStartedState {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            actors: Vec::new(),
            actor_pointer: 0,
            map: Default::default(),
            current_turn: None,
            waiting: false,
            callbacks: Channel::new(5),
        })
    }

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
        for addr in host.get_clients() {
            host.send(
                addr,
                ServerMessage::SpawnActor {
                    name: actor.name.clone(),
                    id,
                    x,
                    y,
                    abilities: actor.abilities.get_keys(),
                    yours: actor.controller == Controller::Player(addr),
                },
            )
            .await?;
        }
        self.actors.push(actor);

        Ok(())
    }

    pub async fn next_actor(&mut self, host: &mut NetworkHost) -> anyhow::Result<()> {
        self.actor_pointer = (self.actor_pointer + 1) % self.actors.len();
        if let Some(actor) = self.actors.get(self.actor_pointer) {
            let addr = match actor.controller {
                Controller::Player(addr) => Some(addr),
                Controller::Bot => None,
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
        }
        Ok(())
    }

    fn current_actor(&self) -> &Actor {
        &self.actors[self.actor_pointer]
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
            |pos| (pos.distance_f32(to) * 10.0) as usize,
            |a| *a == to,
        )
    }

    fn timer<
        I: FnOnce(&'static mut GameStartedState, Arg<'static>) -> F + Send + 'static,
        F: Future<Output = ()> + Send,
    >(
        &mut self,
        time: Duration,
        func: I,
    ) {
        let sender = self.callbacks.send.clone();
        tokio::spawn(async move {
            tokio::time::sleep(time).await;
            _ = sender
                .send(Box::new(move |state, arg| {
                    Box::pin(async move { func(state, arg).await })
                }))
                .await;
        });
    }

    async fn move_current_actor(&mut self, arg: Arg<'_>, Vec2 { x, y }: Vec2) -> Res {
        let Some(Piece::Empty) = self.map.get(y).and_then(|r| r.get(x)) else {
            self.waiting = false;
            self.next_actor(arg.host).await?;
            return Ok(None);
        };
        let Vec2 { x: old_x, y: old_y } = self.actors[self.actor_pointer].position;
        let path = self
            .pathfind(Vec2::new(old_x, old_y), Vec2::new(x, y))
            .await;
        let Some((path, _)) = path else {
            self.waiting = false;
            self.next_actor(arg.host).await?;
            return Ok(None);
        };
        let time = path.len() as f32 / TILES_PER_SECOND as f32;
        // arg.host
        //     .broadcast(ServerMessage::ChatMessage(
        //         "SERVER".to_string(),
        //         format!("Should take {time}s"),
        //     ))
        //     .await?;
        swap(
            unsafe { std::mem::transmute::<&mut Piece, &'static mut Piece>(&mut self.map[y][x]) },
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
        self.waiting = true;

        self.timer(Duration::from_secs_f32(time), async move |state, arg| {
            state.waiting = false;
            state.next_actor(arg.host).await.unwrap();
            // arg.host
            //     .broadcast(ServerMessage::ChatMessage(
            //         "SERVER".to_string(),
            //         "Timer up!".to_string(),
            //     ))
            //     .await
            //     .unwrap();
        });
        Ok(None)
    }
}
#[async_trait::async_trait]
impl State for GameStartedState {
    async fn host_poll_tick<'l>(&mut self, arg: Arg<'l>) -> Res {
        if let Some(Controller::Bot) = self.actors.get(self.actor_pointer).map(|a| a.controller)
            && !self.waiting
        {
            self.move_current_actor(
                unsafe { arg.clone() },
                Vec2::new(rand::random_range(0..=16), rand::random_range(0..=16)),
            )
            .await?;
        }
        if let Ok(fut) = self.callbacks.recv.try_recv() {
            unsafe {
                fut(
                    std::mem::transmute::<&mut GameStartedState, &mut GameStartedState>(self),
                    std::mem::transmute::<Arg<'_>, Arg<'_>>(arg),
                )
                .await;
            }
        }
        Ok(None)
    }

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
            ClientRequest::Action(act, x, y)
                if Some(Controller::Player(addr)) == self.current_turn
                    && !self.waiting
                    && self.current_actor().abilities.contains(&act) =>
            {
                match &*act {
                    "Walk" => self.move_current_actor(arg, Vec2::new(x, y)).await,
                    _ => {
                        arg.host
                            .send(
                                addr,
                                ServerMessage::ChatMessage(
                                    "SERVER".to_string(),
                                    format!("unknown ability: {act:?}"),
                                ),
                            )
                            .await?;
                        Ok(None)
                    }
                }
            }
            req => self.default_client_request(addr, req, arg).await,
        }
    }
}
