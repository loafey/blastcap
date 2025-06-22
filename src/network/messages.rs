use rkyv::{Archive, Deserialize, Serialize};
use std::net::SocketAddr;

#[repr(C)]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ServerMessage {
    Pong(Vec<SocketAddr>),
    ChatMessage(SocketAddr, String),
    NewUser(SocketAddr),
    UserLeft(SocketAddr),
    Status { user_count: usize, tick_diff: f32 },
}

#[repr(C)]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ClientRequest {
    Ping,
    ChatMessage(String),
}
