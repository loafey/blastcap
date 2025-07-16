use super::Arg;
use crate::{
    constants::TILES_PER_SECOND,
    game::{
        actor::{Actor, Controller},
        map::{Map, Piece},
        state::{Res, State},
    },
    network::{
        NetworkHost, SocketAddrExt,
        channel::Channel,
        messages::{ClientRequest, ServerMessage},
    },
};
use math::Vec3;
use std::{pin::Pin, time::Duration};

type Callback = Box<
    dyn FnOnce(
            &'static mut GameStartedState,
            Arg<'static>,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>
        + Send,
>;
pub struct GameStartedState {
    pub actors: Vec<Actor>,
    pub actor_pointer: usize,
    pub map: Box<Map>,
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
    ) -> anyhow::Result<bool> {
        let Some(Piece::Empty) = self.map.get(actor.position) else {
            return Ok(false);
        };
        if actor.position.y == 0 {
            return Ok(false);
        }
        let Some(Piece::Ground) = self.map.get(Vec3 {
            x: actor.position.x,
            y: actor.position.y - 1,
            z: actor.position.z,
        }) else {
            return Ok(false);
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
                    z: actor.position.z,
                    abilities: actor.abilities.get_keys(),
                    yours: actor.controller == Controller::Player(addr),
                    health: actor.health,
                    max_health: actor.health,
                },
            )
            .await?;
        }
        self.actors.push(actor);

        Ok(true)
    }

    pub async fn next_actor(&mut self, host: &mut NetworkHost) -> anyhow::Result<()> {
        let start = self.actor_pointer;
        let mut first = true;
        while !self.actors.is_empty() && (self.actor_pointer != start || first) {
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

    pub fn current_actor(&self) -> &Actor {
        &self.actors[self.actor_pointer]
    }
    pub fn current_actor_mut(&mut self) -> &mut Actor {
        &mut self.actors[self.actor_pointer]
    }

    pub fn get_neighbors(&self, air: bool, Vec3 { x, y, z }: Vec3) -> Vec<(Piece, Vec3)> {
        let (nx, ny, nz) = (x as isize, y as isize, z as isize);
        let mut neighs = Vec::with_capacity(9);
        for z in -1..=1 {
            for y in -1..=1 {
                for x in -1..=1 {
                    if x == 0 && y == 0 && z == 0 {
                        continue;
                    }
                    let z = (nz + z) as usize;
                    let y = (ny + y) as usize;
                    let x = (nx + x) as usize;
                    let v = Vec3::new(x, y, z);
                    if let Some(piece) = self.map.get(v) {
                        if air {
                            neighs.push((piece, v));
                        } else if let Some(Piece::Ground) = self.map.get(Vec3 {
                            x,
                            y: y.wrapping_sub(1),
                            z,
                        }) {
                            neighs.push((piece, v));
                        }
                    }
                }
            }
        }
        neighs
    }

    pub fn pathfind(&self, from: Vec3, to: Vec3) -> Option<(Vec<Vec3>, usize)> {
        pathfinding::directed::astar::astar(
            &from,
            |p| {
                self.get_neighbors(false, *p)
                    .into_iter()
                    .filter_map(|(p, v)| {
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
        pos: Vec3,
    ) -> anyhow::Result<Option<Duration>> {
        let Some(Piece::Empty) = self.map.get(pos) else {
            self.waiting = false;
            return Ok(None);
        };
        let old = self.actors[self.actor_pointer].position;
        let path = self.pathfind(old, pos);
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
        let mut z_list = Vec::new();
        for Vec3 { x, y, z } in path {
            x_list.push(x);
            y_list.push(y);
            z_list.push(z);
        }
        arg.host
            .broadcast(ServerMessage::MoveActor {
                actor: self.actor_pointer,
                x: x_list,
                y: y_list,
                z: z_list,
            })
            .await?;
        self.waiting = true;

        Ok(Some(Duration::from_secs_f32(time)))
    }

    async fn current_punch_actor(
        &mut self,
        arg: Arg<'_>,
        pos: Vec3,
    ) -> anyhow::Result<Option<Duration>> {
        let cur_act = self.current_actor();
        if cur_act.resources.abilities == 0 {
            return Ok(None);
        }
        let actor_pos = cur_act.position;
        let distance = actor_pos.distance(pos);
        if distance > 1 {
            return Ok(None);
        }
        let hit = self.map.get(pos);
        let Some(Piece::Actor(hit_ptr)) = hit else {
            return Ok(None);
        };
        if hit_ptr == self.actor_pointer {
            return Ok(None);
        }
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
        self.current_actor_mut().resources.abilities -= 1;
        self.waiting = true;
        Ok(Some(Duration::from_secs_f32(0.5)))
    }
}
#[async_trait::async_trait]
impl State for GameStartedState {
    async fn host_poll_tick<'l>(&mut self, arg: Arg<'l>) -> Res {
        let curr_act = self.actors.get(self.actor_pointer);
        if let Some(actor) = curr_act
            && matches!(actor.controller, Controller::Bot)
            && !self.waiting
        {
            actor.bot_act(self, unsafe { arg.clone() }).await?;
        }

        while let Ok(fut) = self.callbacks.recv.try_recv() {
            unsafe {
                fut(
                    std::mem::transmute::<&mut GameStartedState, &mut GameStartedState>(self),
                    std::mem::transmute::<Arg<'_>, Arg<'_>>(arg.clone()),
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
            ClientRequest::Action(act, x, y, z)
                if (Some(Controller::Player(addr)) == self.current_turn || addr.is_bot())
                    && !self.waiting
                    && self.current_actor().abilities.contains(&act) =>
            {
                match &*act {
                    "Walk" => {
                        if let Some(t) = self.move_current_actor(arg, Vec3::new(x, y, z)).await? {
                            self.timer(t, async |s, _| {
                                s.waiting = false;
                                Ok(())
                            });
                        }
                        Ok(None)
                    }
                    "Punch" => {
                        if let Some(t) = self.current_punch_actor(arg, Vec3::new(x, y, z)).await? {
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
                if (Some(Controller::Player(addr)) == self.current_turn || addr.is_bot())
                    && !self.waiting =>
            {
                self.next_actor(arg.host).await?;
                self.current_actor_mut().reset_turn_resources();
                Ok(None)
            }
            req => self.default_client_request(addr, req, arg).await,
        }
    }
}
