use serde::Deserialize;

use crate::game::actor::Actor;

#[derive(Debug, Deserialize, Default)]
pub struct TurnResources {
    pub abilities: usize,
    pub bonus_actions: usize,
    pub movement: usize,
}
impl Actor {
    pub fn reset_turn_resources(&mut self) {
        self.resources = TurnResources {
            abilities: 1,
            bonus_actions: 1,
            movement: 10,
        };
    }
}
