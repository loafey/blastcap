use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

#[repr(C, i32)]
#[sharpify::client_poll]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum ServerMessage {
    Pong,
    ChatMessage(u64, String),
    NewUser(u64),
    UserLeft(u64),
    PlayerList(Vec<u64>),
    Status {
        user_count: u32,
        tick_diff: f32,
    },
    NotifyHost,
    MapList(Vec<String>),
    StartMap(String),
    SpawnActor {
        yours: bool,
        name: String,
        id: usize,
        x: usize,
        y: usize,
        z: usize,
        abilities: Vec<String>,
        movement: u32,
        health: i32,
        max_health: i32,
    },
    YourTurn {
        actor: usize,
        movement: u32,
    },
    ActorTurn {
        actor: usize,
    },
    MoveActor {
        actor: usize,
        movement: u32,
        x: Vec<usize>,
        y: Vec<usize>,
        z: Vec<usize>,
    },
    AbilityMap(HashMap<String, String>),
    Action {
        action: String,
        actor: usize,
        target: usize,
        target_damage: i32,
        time: f32,
    },
    SpawnMap {
        x: Vec<usize>,
        y: Vec<usize>,
        z: Vec<usize>,
    },
    EnterDungeonState,
}

#[repr(C)]
#[sharpify::client_interface]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ClientRequest {
    Ping,
    ChatMessage(String),
    RequestMapList,
    StartMap(String),
    NotifyReady,
    Action(String, usize, usize, usize),
    EndTurn,
    ChangeDungeonSetting(u32, u32),
    ChangeToEnterDungeon,
}
