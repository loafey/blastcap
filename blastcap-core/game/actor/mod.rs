use math::Vec2;
use serde::Deserialize;
use std::net::SocketAddr;

mod resources;
pub use resources::*;

mod abilities;
pub use abilities::*;

use crate::{
    game::{
        Arg,
        state::{GameStartedState, Piece},
    },
    network::messages::ClientRequest,
};

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
    pub resources: TurnResources,
}
impl Actor {
    pub async fn bot_act<'l>(&self, state: &GameStartedState, arg: Arg<'l>) -> anyhow::Result<()> {
        let neighs = state
            .get_neighbors(self.position)
            .into_iter()
            .filter_map(|(f, p)| match f {
                Piece::Actor(i) if state.actors[i].health > 0 => Some(p),
                _ => None,
            })
            .collect::<Vec<_>>();
        if self.resources.abilities == 0 {
            arg.host.mock(ClientRequest::EndTurn).await?;
        } else if !neighs.is_empty() && rand::random_range(0..=1) == 0 {
            let pos = neighs[rand::random_range(0..neighs.len())];
            arg.host
                .mock(ClientRequest::Action("Punch".to_string(), pos.x, pos.y))
                .await?
        } else {
            arg.host
                .mock(ClientRequest::Action(
                    "Walk".to_string(),
                    rand::random_range(0..=16),
                    rand::random_range(0..=16),
                ))
                .await?
        };
        Ok(())
    }
}
