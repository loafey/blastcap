use math::Vec3;
use rand::seq::IndexedRandom;
use serde::Deserialize;

mod resources;
pub use resources::*;

mod abilities;
pub use abilities::*;

use crate::{
    game::{Arg, map::Piece, state::GameStartedState},
    network::messages::ClientRequest,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Controller {
    Player(u64),
    #[default]
    Bot,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Actor {
    pub name: String,
    #[serde(skip)]
    pub controller: Controller,
    #[serde(skip)]
    pub position: Vec3,
    pub health: i32,
    pub base_movement: u32,
    pub abilities: Abilities,
    pub resources: TurnResources,
}
impl Actor {
    pub async fn bot_act<'l>(&self, state: &GameStartedState, arg: Arg<'l>) -> anyhow::Result<()> {
        let neighs = state
            .get_neighbors(false, self.position)
            .into_iter()
            .filter_map(|(f, p)| match f {
                Piece::Actor(i) if state.actors[i].health > 0 => Some(p),
                _ => None,
            })
            .collect::<Vec<_>>();
        if self.resources.abilities == 0 {
            arg.host.mock(ClientRequest::EndTurn).await?;
        } else if !neighs.is_empty()
        /*&& rand::random_range(0..=1) == 0*/
        {
            let pos = neighs[rand::random_range(0..neighs.len())];
            arg.host
                .mock(ClientRequest::Action(
                    "Punch".to_string(),
                    pos.x,
                    pos.y,
                    pos.z,
                ))
                .await?
        } else {
            let others = state
                .actors
                .iter()
                .filter(|a| a.position != self.position && a.health > 0)
                .collect::<Vec<_>>();
            if others.is_empty() {
                arg.host.mock(ClientRequest::EndTurn).await?;
                return Ok(());
            }
            let random = others[rand::random_range(0..others.len())];
            let Some((Vec3 { x, y, z }, _, _)) = state
                .get_neighbors(false, random.position)
                .into_iter()
                .filter(|(a, _)| matches!(a, Piece::Empty))
                .map(|a| a.1)
                .filter_map(|a| state.pathfind(self.position, a).map(|(b, c)| (a, b, c)))
                .min_by_key(|(_, _, c)| *c)
            else {
                arg.host.mock(ClientRequest::EndTurn).await?;
                return Ok(());
            };

            arg.host
                .mock(ClientRequest::ChatMessage(format!(
                    "{} targeting {}",
                    self.name, random.name
                )))
                .await?;

            arg.host
                .mock(ClientRequest::Action("Walk".to_string(), x, y, z))
                .await?
        };
        Ok(())
    }
}
