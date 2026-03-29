use rand::seq::SliceRandom;
use serde::Deserialize;

use crate::game_data::DATA;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CardHolder {
    hand: Vec<u64>,
    cards: Vec<u64>,
    trash: Vec<u64>,
}
impl CardHolder {
    pub fn draw(&mut self, amount: usize) -> Vec<String> {
        let mut data = Vec::new();
        for _ in 0..amount {
            let Some(card) = self.cards.pop() else { break };
            let Some(card_data) = DATA.cards.get(&card) else {
                error!("tried to fetch a card that does not exist");
                continue;
            };
            data.push(card_data.name.clone());
            self.hand.push(card);
        }
        data
    }

    pub fn test_data() -> Self {
        let mut cards = DATA.cards.clone().keys().copied().collect::<Vec<_>>();
        let clone = cards.clone();
        cards.append(&mut clone.clone());
        cards.append(&mut clone.clone());
        cards.append(&mut clone.clone());
        cards.shuffle(&mut rand::rng());
        Self {
            hand: Vec::new(),
            cards,
            trash: Vec::new(),
        }
    }
}
