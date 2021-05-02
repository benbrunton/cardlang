use super::*;
use std::fmt;

pub struct Player {
    id: i32,
    _hand: Vec<Card>
}

impl Player{
    pub fn new(id: i32) -> Player {
        Player { _hand: vec!(), id }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "player {}", self.id)
    }
}