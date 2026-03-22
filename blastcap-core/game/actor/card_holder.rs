use serde::Deserialize;

use crate::game_data::DATA;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CardHolder {
    hand: Vec<data::Card>,
    cards: Vec<data::Card>,
    trash: Vec<data::Card>,
}
impl CardHolder {
    pub fn test_data() -> Self {
        Self {
            hand: Vec::new(),
            cards: DATA.cards.clone(),
            trash: Vec::new(),
        }
    }
}
