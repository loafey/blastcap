use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

#[repr(C, i32)]
#[sharpify::client_poll]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ServerMessage {
    Pong,
    ChatMessage(String, String),
    NewUser(String),
    UserLeft(String),
    PlayerList(Vec<String>),
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
        abilities: Vec<String>,
    },
    YourTurn {
        actor: usize,
    },
    ActorTurn {
        actor: usize,
    },
    MoveActor {
        actor: usize,
        x: Vec<usize>,
        y: Vec<usize>,
    },
    AbilityMap(HashMap<String, String>),
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
    MoveActor(usize, usize),
}
