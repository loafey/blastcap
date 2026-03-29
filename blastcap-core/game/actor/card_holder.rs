use data::Card;
use rand::seq::SliceRandom;
use serde::Deserialize;

use crate::game_data::DATA;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CardHolder {
    hand: Vec<Card>,
    cards: Vec<Card>,
    trash: Vec<Card>,
}
impl CardHolder {
    pub fn draw(&mut self, amount: usize) -> Vec<String> {
        let mut data = Vec::new();
        for _ in 0..amount {
            let Some(card) = self.cards.pop() else { break };
            data.push(card.name.clone());
            self.hand.push(card);
        }
        data
    }

    pub fn test_data() -> Self {
        let mut cards = DATA.cards.clone();
        cards.append(&mut DATA.cards.clone());
        cards.append(&mut DATA.cards.clone());
        cards.append(&mut DATA.cards.clone());
        cards.shuffle(&mut rand::rng());
        Self {
            hand: Vec::new(),
            cards,
            trash: Vec::new(),
        }
    }
}
