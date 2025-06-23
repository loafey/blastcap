use rkyv::{Archive, Deserialize, Serialize};

#[repr(C, i32)]
#[sharpify::client_poll]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ServerMessage {
    Pong,
    ChatMessage(String, String),
    NewUser(String),
    UserLeft(String),
    Status { user_count: u32, tick_diff: f32 },
}

#[repr(C)]
#[sharpify::client_interface]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum ClientRequest {
    Ping,
    ChatMessage(String),
}
