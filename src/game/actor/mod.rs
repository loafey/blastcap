use math::Vec2;
use std::net::SocketAddr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Controller {
    Player(SocketAddr),
}

pub struct Actor {
    pub name: String,
    pub id: usize,
    pub controller: Controller,
    pub position: Vec2,
}
