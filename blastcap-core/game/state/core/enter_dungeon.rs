use std::{collections::HashSet, time::Duration};

use math::Vec3;

use crate::{
    game::{
        Arg,
        actor::{Actor, CardHolder, Controller},
        state::{ClearRoomState, Res, State},
    },
    network::messages::{ClientRequest, ServerMessage},
};

enum DungeonSettings {
    BotAmount,
}
impl TryFrom<u32> for DungeonSettings {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use DungeonSettings::*;
        match value {
            0 => Ok(BotAmount),
            _ => Err(()),
        }
    }
}

pub struct EnterDungeonState {
    waiting_for: HashSet<u64>,
    players: HashSet<u64>,
    bot_amount: u32,
}
impl EnterDungeonState {
    pub fn new<I: IntoIterator<Item = u64>>(waiting_for: I) -> Box<Self> {
        Box::new(Self {
            waiting_for: HashSet::from_iter(waiting_for),
            players: Default::default(),
            bot_amount: 0,
        })
    }
}
#[async_trait::async_trait]
impl State for EnterDungeonState {
    async fn client_req<'l>(&mut self, addr: u64, req: ClientRequest, arg: Arg<'l>) -> Res {
        match req {
            ClientRequest::ChangeDungeonSetting(setting, value)
                if Some(addr) == arg.data.host_player =>
            {
                let setting = match DungeonSettings::try_from(setting) {
                    Ok(s) => s,
                    Err(_) => {
                        error!("unknown setting {setting}");
                        return Ok(None);
                    }
                };
                match setting {
                    DungeonSettings::BotAmount => {
                        arg.host
                            .broadcast(ServerMessage::ServerNotice(format!(
                                "Bot amount: {} -> {}",
                                self.bot_amount, value
                            )))
                            .await?;
                        self.bot_amount = value
                    }
                }
                //
                Ok(None)
            }
            ClientRequest::NotifyReady(ready) => {
                arg.host
                    .broadcast(ServerMessage::ReadyStatus(addr, ready))
                    .await?;
                let ready = ready != 0;
                if !ready {
                    self.players.remove(&addr);
                    self.waiting_for.insert(addr);
                    Ok(None)
                } else {
                    if self.waiting_for.remove(&addr) {
                        self.players.insert(addr);
                    }
                    if self.waiting_for.is_empty() {
                        arg.host
                            .broadcast(ServerMessage::EnterClearRoomState)
                            .await?;
                        // TODO: Make player send ready checks
                        smol::Timer::after(Duration::from_secs(1)).await;
                        trace!(
                            "Starting game with player actor controllers: {:?}",
                            self.players
                        );

                        let mut gs = ClearRoomState::new(Vec3::new(80, 40, 80), |m| {
                            m.gen_sparse_floor(0, 3);
                            // m.gen_sparse_floor(10, 3);
                            // m.gen_sparse_floor(20, 3);
                        });
                        let (x_list, y_list, z_list) = gs.map.get_ground_data();

                        arg.host
                            .broadcast(ServerMessage::SpawnMap {
                                x: x_list,
                                y: y_list,
                                z: z_list,
                            })
                            .await?;
                        let map_size = gs.map.get_size();
                        for (id, addr) in self.players.iter().copied().enumerate() {
                            let actor = Actor {
                                name: format!("Player {id}"),
                                controller: Controller::Player(addr),
                                position: Vec3::new(15, 1, 15),
                                abilities: Default::default(),
                                health: 10,
                                base_movement: u32::MAX, //rand::random_range(10..20),
                                resources: Default::default(),
                                cards: CardHolder::test_data(),
                            };
                            while {
                                let position = Vec3::new(
                                    rand::random_range(0..map_size.x),
                                    1,
                                    rand::random_range(0..map_size.z),
                                );
                                let clone = actor.clone();
                                let res = gs
                                    .spawn_actor(arg.host, Actor { position, ..clone })
                                    .await?;
                                !res
                            } {}
                        }
                        info!("Players spawned");
                        let mut i = 0;
                        while i < self.bot_amount {
                            let position = Vec3::new(
                                rand::random_range(0..map_size.x),
                                1,
                                rand::random_range(0..map_size.z),
                            );

                            let mut actor = Actor {
                                name: format!("Bot {i}"),
                                controller: Controller::Bot,
                                position,
                                health: 15,
                                base_movement: rand::random_range(10..20),
                                abilities: Default::default(),
                                resources: Default::default(),
                                cards: Default::default(),
                            };
                            actor.reset_turn_resources();
                            if gs.spawn_actor(arg.host, actor).await? {
                                i += 1;
                            }
                        }
                        info!("Bots spawned");
                        gs.next_actor(arg.host).await?;
                        arg.host
                            .broadcast(ServerMessage::ActorList {
                                names: gs.actors.iter().map(|a| a.name.clone()).collect(),
                            })
                            .await?;
                        Ok(Some(gs))
                    } else {
                        Ok(None)
                    }
                }
            }
            req => self.default_client_request(addr, req, arg).await,
        }
    }
}
