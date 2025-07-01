use math::Vec2;
use std::net::SocketAddr;

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
}
