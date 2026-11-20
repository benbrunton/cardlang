use super::*;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Player {
    id: u32,
    hand: Vec<Card>,
    // stacks declared with `stack <name> per player`
    stacks: HashMap<String, Vec<Card>>
}

impl Player{
    pub fn with_stacks(id: u32, stack_names: &[String]) -> Player {
        let stacks = stack_names.iter().map(|name| (name.clone(), vec!())).collect();
        Player { hand: vec!(), id, stacks }
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
