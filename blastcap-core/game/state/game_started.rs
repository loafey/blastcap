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
use std::{pin::Pin, time::Duration};

#[derive(Default)]
struct Map {
    alive: [[Piece; 16]; 16],
    dead: [[Option<usize>; 16]; 16],
}
impl Map {
    pub fn get(&self, Vec2 { x, y }: Vec2) -> Option<Piece> {
        self.alive.get(y).and_then(|r| r.get(x).copied())
    }

    pub fn set(&mut self, Vec2 { x, y }: Vec2, value: Piece) {
        let Some(piece) = self.alive.get_mut(y).and_then(|r| r.get_mut(x)) else {
            return;
        };
        *piece = value;
    }

    pub fn dead_get(&self, Vec2 { x, y }: Vec2) -> Option<usize> {
        self.dead
            .get(y)
            .and_then(|r| r.get(x).copied().and_then(|r| r))
    }

    pub fn dead_set(&mut self, Vec2 { x, y }: Vec2, value: Option<usize>) {
        let Some(piece) = self.dead.get_mut(y).and_then(|r| r.get_mut(x)) else {
            return;
        };
        *piece = value;
    }
}
#[derive(Default, Debug, Clone, Copy)]
enum Piece {
    #[default]
    Empty,
    Actor(usize),
}

type Callback = Box<
    dyn FnOnce(
            &'static mut GameStartedState,
            Arg<'static>,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>
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
        let Some(Piece::Empty) = self.map.get(actor.position) else {
            return Ok(());
        };

        let id = self.actors.len();
        self.map.set(actor.position, Piece::Actor(id));
        for addr in host.get_clients() {
            host.send(
                addr,
                ServerMessage::SpawnActor {
                    name: actor.name.clone(),
                    id,
                    x: actor.position.x,
                    y: actor.position.y,
                    abilities: actor.abilities.get_keys(),
                    yours: actor.controller == Controller::Player(addr),
                    health: actor.health,
                    max_health: actor.health,
                },
            )
            .await?;
        }
        self.actors.push(actor);

        Ok(())
    }

    pub async fn next_actor(&mut self, host: &mut NetworkHost) -> anyhow::Result<()> {
        let start = self.actor_pointer;
        let mut first = true;
        while self.actor_pointer != start || first {
            first = false;
            self.actor_pointer = (self.actor_pointer + 1) % self.actors.len();
            if let Some(actor) = self.actors.get(self.actor_pointer) {
                if actor.health <= 0 {
                    continue;
                }
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
                break;
            }
        }
        Ok(())
    }

    fn current_actor(&self) -> &Actor {
        &self.actors[self.actor_pointer]
    }

    fn get_neighbors(&self, Vec2 { x, y }: Vec2) -> Vec<(Piece, Vec2)> {
        let (nx, ny) = (x as isize, y as isize);
        let mut neighs = Vec::with_capacity(9);
        for y in -1..=1 {
            for x in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }
                let y = (ny + y) as usize;
                let x = (nx + x) as usize;
                if let Some(piece) = self.map.get(Vec2::new(x, y)) {
                    neighs.push((piece, Vec2::new(x, y)));
                }
            }
        }
        neighs
    }

    async fn pathfind(&self, from: Vec2, to: Vec2) -> Option<(Vec<Vec2>, usize)> {
        pathfinding::directed::astar::astar(
            &from,
            |p| {
                self.get_neighbors(*p).into_iter().filter_map(|(p, v)| {
                    if let Piece::Empty = p {
                        Some((v, 1))
                    } else {
                        None
                    }
                })
            },
            |pos| (pos.distance_f32(to) * 10.0) as usize,
            |a| *a == to,
        )
    }

    fn timer<
        I: FnOnce(&'static mut GameStartedState, Arg<'static>) -> F + Send + 'static,
        F: Future<Output = anyhow::Result<()>> + Send,
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

    async fn move_current_actor(
        &mut self,
        arg: Arg<'_>,
        pos: Vec2,
    ) -> anyhow::Result<Option<Duration>> {
        let Some(Piece::Empty) = self.map.get(pos) else {
            self.waiting = false;
            return Ok(None);
        };
        let old = self.actors[self.actor_pointer].position;
        let path = self.pathfind(old, pos).await;
        let Some((path, _)) = path else {
            self.waiting = false;
            return Ok(None);
        };
        let time = path.len() as f32 / TILES_PER_SECOND as f32;
        {
            let a = self.map.get(pos).unwrap();
            let b = self.map.get(old).unwrap();
            self.map.set(pos, b);
            self.map.set(old, a);
        }
        self.actors[self.actor_pointer].position = pos;
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

        Ok(Some(Duration::from_secs_f32(time)))
    }

    async fn current_punch_actor(
        &mut self,
        arg: Arg<'_>,
        pos: Vec2,
    ) -> anyhow::Result<Option<Duration>> {
        let actor_pos = self.current_actor().position;
        let distance = actor_pos.distance(pos);
        let hit = self.map.get(pos);
        if distance > 1 {
            return Ok(None);
        }
        let Some(Piece::Actor(hit_ptr)) = hit else {
            return Ok(None);
        };
        let Some(hit) = self.actors.get(hit_ptr) else {
            return Ok(None);
        };

        let dmg = 15;
        arg.host
            .broadcast(ServerMessage::ChatMessage(
                self.current_actor().name.clone(),
                format!("punched {:?} at {distance}B", hit.name),
            ))
            .await?;
        arg.host
            .broadcast(ServerMessage::Action {
                action: "Punch".to_string(),
                actor: self.actor_pointer,
                target: hit_ptr,
                target_damage: dmg,
                time: 0.5,
            })
            .await?;
        if let Some(hit) = self.actors.get_mut(hit_ptr) {
            hit.health -= dmg;
            if hit.health <= 0 {
                let Some(Piece::Actor(id)) = self.map.get(hit.position) else {
                    unreachable!()
                };
                self.map.set(hit.position, Piece::Empty);
                self.map.dead_set(hit.position, Some(id));
            }
        };
        self.waiting = true;
        Ok(Some(Duration::from_secs_f32(0.5)))
    }
}
#[async_trait::async_trait]
impl State for GameStartedState {
    async fn host_poll_tick<'l>(&mut self, arg: Arg<'l>) -> Res {
        if let Some(Controller::Bot) = self.actors.get(self.actor_pointer).map(|a| a.controller)
            && !self.waiting
        {
            let neighs = self
                .get_neighbors(self.current_actor().position)
                .into_iter()
                .filter_map(|(f, p)| match f {
                    Piece::Actor(i) if self.actors[i].health > 0 => Some(p),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let time = if !neighs.is_empty() && rand::random_range(0..=1) == 0 {
                let pos = neighs[rand::random_range(0..neighs.len())];
                self.current_punch_actor(unsafe { arg.clone() }, pos)
                    .await?
            } else {
                self.move_current_actor(
                    unsafe { arg.clone() },
                    Vec2::new(rand::random_range(0..=16), rand::random_range(0..=16)),
                )
                .await?
            };
            if let Some(time) = time {
                self.timer(time, async |state, arg| {
                    state.waiting = false;
                    state.next_actor(arg.host).await?;
                    Ok(())
                });
            }
        }

        if let Ok(fut) = self.callbacks.recv.try_recv() {
            unsafe {
                fut(
                    std::mem::transmute::<&mut GameStartedState, &mut GameStartedState>(self),
                    std::mem::transmute::<Arg<'_>, Arg<'_>>(arg),
                )
                .await?;
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
                    "Walk" => {
                        if let Some(t) = self.move_current_actor(arg, Vec2::new(x, y)).await? {
                            self.timer(t, async |s, _| {
                                s.waiting = false;
                                Ok(())
                            });
                        }
                        Ok(None)
                    }
                    "Punch" => {
                        if let Some(t) = self.current_punch_actor(arg, Vec2::new(x, y)).await? {
                            self.timer(t, async |s, _| {
                                s.waiting = false;
                                Ok(())
                            });
                        }
                        Ok(None)
                    }
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
            ClientRequest::EndTurn
                if Some(Controller::Player(addr)) == self.current_turn && !self.waiting =>
            {
                self.next_actor(arg.host).await?;
                Ok(None)
            }
            req => self.default_client_request(addr, req, arg).await,
        }
    }
}
