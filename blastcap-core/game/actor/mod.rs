use math::Vec2;
use serde::Deserialize;
use std::net::SocketAddr;

mod abilities;
pub use abilities::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Controller {
    Player(SocketAddr),
    #[default]
    Bot,
}

#[derive(Debug, Deserialize)]
pub struct Actor {
    pub name: String,
    #[serde(skip)]
    pub controller: Controller,
    #[serde(skip)]
    pub position: Vec2,
    pub health: i32,
    pub base_movement: usize,
    pub abilities: Abilities,
}
