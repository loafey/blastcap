use crate::network::{
    HostPoll, NetworkHost, TICK_RATE,
    messages::{ClientRequest, ServerMessage},
};
use std::{
    collections::{HashSet, VecDeque},
    net::SocketAddr,
    time::Instant,
};

mod state;

type Map = [[u8; 16]; 16];
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Controller {
    Player(SocketAddr),
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
struct Vec2 {
    x: usize,
    y: usize,
}
impl Vec2 {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}
struct Actor {
    name: String,
    id: usize,
    controller: Controller,
    position: Vec2,
}

struct GameStarted {
    id_counter: usize,
    actors: VecDeque<Actor>,
    map: Box<Map>,
    current_turn: Option<Controller>,
    current_id: usize,
}
impl GameStarted {
    async fn next_actor(&mut self, host: &mut NetworkHost) -> anyhow::Result<()> {
        if let Some(actor) = self.actors.pop_front() {
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
            self.actors.push_back(actor);
        }
        Ok(())
    }
}

#[derive(Default)]
enum State {
    #[default]
    Lobby,
    Waiting {
        waiting_for: HashSet<SocketAddr>,
        players: HashSet<SocketAddr>,
    },
    GameStarted(GameStarted),
}

#[derive(Default)]
struct ServerData {
    host_player: Option<SocketAddr>,
    tick: usize,
}

pub async fn host_loop(port: u16) -> anyhow::Result<()> {
    let mut host = NetworkHost::tcp(port).await?;

    let mut data = ServerData::default();
    let mut state = State::Lobby;
    let mut last_tick = Instant::now();
    while let Ok(res) = host.poll().await {
        match res {
            HostPoll::ClientConnected(addr) => {
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
            }
            HostPoll::ClientRequest { addr, req } => match req {
                ClientRequest::Ping => host.send(addr, ServerMessage::Pong).await?,
                ClientRequest::ChatMessage(msg) => {
                    host.broadcast(ServerMessage::ChatMessage(format!("{addr}"), msg))
                        .await?;
                }
                ClientRequest::RequestMapList => {
                    if Some(addr) != data.host_player {
                        continue;
                    };
                    host.send(
                        addr,
                        ServerMessage::MapList(vec!["SimpleTestMap".to_string()]),
                    )
                    .await?;
                }
                ClientRequest::StartMap(map) => {
                    if Some(addr) != data.host_player {
                        continue;
                    };
                    state = State::Waiting {
                        waiting_for: HashSet::from_iter(host.get_clients()),
                        players: HashSet::new(),
                    };

                    host.broadcast(ServerMessage::StartMap(map)).await?;
                }
                ClientRequest::NotifyReady => {
                    let State::Waiting {
                        waiting_for,
                        players,
                    } = &mut state
                    else {
                        continue;
                    };
                    if waiting_for.remove(&addr) {
                        players.insert(addr);
                    }

                    if waiting_for.is_empty() {
                        println!(
                            "SERVER - Starting game with player actor controllers: {players:?}"
                        );

                        let mut posses = [
                            Vec2::new(0, 0),
                            Vec2::new(0, 15),
                            Vec2::new(15, 0),
                            Vec2::new(15, 15),
                        ]
                        .into_iter()
                        .cycle();
                        let waiting_actors = players
                            .iter()
                            .copied()
                            .enumerate()
                            .map(|(id, addr)| Actor {
                                name: format!("Player {id}"),
                                id,
                                controller: Controller::Player(addr),
                                position: posses.next().unwrap(),
                            })
                            .collect::<Vec<_>>();
                        for wa in &waiting_actors {
                            host.broadcast(ServerMessage::SpawnPlayer {
                                name: wa.name.clone(),
                                id: wa.id,
                                x: wa.position.x,
                                y: wa.position.y,
                            })
                            .await?;
                        }
                        let mut gs = GameStarted {
                            id_counter: waiting_actors.len(),
                            actors: VecDeque::from(waiting_actors),
                            map: Default::default(),
                            current_turn: None,
                            current_id: usize::MAX,
                        };
                        gs.next_actor(&mut host).await?;
                        state = State::GameStarted(gs);
                    }
                }
                ClientRequest::MoveActor(x, y) => {
                    let State::GameStarted(gs) = &mut state else {
                        continue;
                    };
                    if Some(Controller::Player(addr)) != gs.current_turn {
                        continue;
                    };

                    println!("UPDATE ON SERVER!!! {x} {y}");
                    host.broadcast(ServerMessage::MoveActor {
                        actor: gs.current_id,
                        x,
                        y,
                    })
                    .await?;
                    gs.next_actor(&mut host).await?;
                }
            },
            HostPoll::Tick => {
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
                    last_tick = Instant::now()
                }
            }
            HostPoll::RemoveClient(socket_addr) => {
                host.remove_client(socket_addr);
                host.broadcast(ServerMessage::UserLeft(format!("{socket_addr}")))
                    .await?;
            }
        }
    }
    Ok(())
}
