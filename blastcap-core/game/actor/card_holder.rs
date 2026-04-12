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
    pub fn trash_card(&mut self, index: usize) {
        if self.hand.len() >= index {
            return;
        }
        let card = self.hand.remove(index);
        self.trash.push(card);
    }

    pub fn check_hand(&self, index: usize) -> Option<u64> {
        self.hand.get(index).copied()
    }

    pub fn draw(&mut self, amount: usize) -> Vec<u64> {
        let mut data = Vec::new();
        for _ in 0..amount {
            let Some(card) = self.cards.pop() else { break };
            data.push(card);
            self.hand.push(card);
        }
        data
    }

    pub fn test_data() -> Self {
        let mut cards = DATA
            .cards
            .clone()
            .iter()
            .filter(|(_, c)| c.unique_id.is_none())
            .map(|(i, _)| *i)
            .collect::<Vec<_>>();
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
