use rkyv::{Archive, Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ServerMessage {
    Pong(Vec<SocketAddr>),
    ChatMessage(SocketAddr, String),
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ClientRequest {
    Ping,
    ChatMessage(String),
}
