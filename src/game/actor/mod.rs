use math::Vec2;
use std::{collections::HashSet, net::SocketAddr};

mod abilities;
pub use abilities::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Controller {
    Player(SocketAddr),
    Bot,
}

#[derive(Debug)]
pub struct Actor {
    pub name: String,
    pub controller: Controller,
    pub position: Vec2,
    pub abilities: Abilities,
}
