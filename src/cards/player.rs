use super::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Player {
    id: u32,
    hand: Vec<Card>
}

impl Player{
    pub fn new(id: u32) -> Player {
        Player { hand: vec!(), id }
    }

    pub fn get_hand(&self) -> Vec<Card> {
        self.hand.clone()
    }

    pub fn set_hand(&mut self, hand: Vec<Card>) {
        self.hand = hand;
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "player {} (cards: {})", self.id, self.hand.len())
    }
}