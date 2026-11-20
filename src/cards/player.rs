use super::*;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Player {
    id: u32,
    hand: Vec<Card>,
    // stacks declared with `player stack <name>`
    stacks: HashMap<String, Vec<Card>>,
    // numbers declared with `player number <name>`, starting at 0
    numbers: HashMap<String, f64>
}

impl Player{
    pub fn with_fields(id: u32, stack_names: &[String], number_names: &[String]) -> Player {
        let stacks = stack_names.iter().map(|name| (name.clone(), vec![])).collect();
        let numbers = number_names.iter().map(|name| (name.clone(), 0.0)).collect();
        Player { hand: vec![], id, stacks, numbers }
    }

    pub fn get_number(&self, name: &str) -> Option<f64> {
        self.numbers.get(name).copied()
    }

    // returns false if the player has no number with this name
    pub fn set_number(&mut self, name: &str, value: f64) -> bool {
        match self.numbers.get_mut(name) {
            Some(n) => {
                *n = value;
                true
            },
            None => false
        }
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

    // `hand` or a per-player stack
    pub fn get_stack(&self, name: &str) -> Option<Vec<Card>> {
        if name == "hand" {
            return Some(self.get_hand());
        }
        self.stacks.get(name).cloned()
    }

    // returns false if the player has no stack with this name
    pub fn set_stack(&mut self, name: &str, cards: Vec<Card>) -> bool {
        if name == "hand" {
            self.hand = cards;
            return true;
        }
        match self.stacks.get_mut(name) {
            Some(stack) => {
                *stack = cards;
                true
            },
            None => false
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "player {} (cards: {})", self.id, self.hand.len())
    }
}
