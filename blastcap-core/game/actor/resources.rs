use serde::Deserialize;

use crate::game::actor::Actor;

#[derive(Debug, Deserialize, Default)]
pub struct TurnResources {
    pub abilities: usize,
    pub _bonus_actions: usize,
    pub _movement: usize,
}
impl Actor {
    pub fn reset_turn_resources(&mut self) {
        self.resources = TurnResources {
            abilities: 1,
            _bonus_actions: 1,
            _movement: 10,
        };
    }
}
