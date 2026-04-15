use rkyv::{Archive, Deserialize, Serialize};

#[repr(C, i32)]
#[sharpify::client_poll]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum ServerMessage {
    Pong,
    ChatMessage(u64, String),
    ServerNotice(String),
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
        x: i64,
        y: i64,
        z: i64,
        movement: u32,
        health: i32,
        max_health: i32,
    },
    YourTurn {
        actor: usize,
        movement: u32,
        abilities: Vec<u64>,
        cards: Vec<u64>,
    },
    ActorTurn {
        actor: usize,
    },
    MoveActor {
        actor: usize,
        movement: u32,
        x: Vec<i64>,
        y: Vec<i64>,
        z: Vec<i64>,
    },
    Action {
        action: u64,
        actor: usize,
        target: usize,
    },
    GenerateMap {
        seed: u64,
    },
    EnterDungeonState,
    ReadyStatus(u64, u8),
    EnterClearRoomState,
    ActorList {
        names: Vec<String>,
    },
    RemoveCardFromHand(usize),
    // Loading messages
    GameLoadingTotal(usize),
    GameLoadingCard(u64, data::types::Card),
}

#[repr(C)]
#[sharpify::client_interface]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ClientRequest {
    Ping,
    ChatMessage(String),
    RequestMapList,
    StartMap(String),
    NotifyReady(u8),
    Action(usize, i64, i64, i64),
    UseCard(usize, i64, i64, i64),
    EndTurn,
    ChangeDungeonSetting(u32, u32),
    ChangeToEnterDungeon,
}
