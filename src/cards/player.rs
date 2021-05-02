use super::*;

pub struct Player {
    hand: Vec<Card>
}

impl Player{
    pub fn new() -> Player {
        Player { hand: vec!() }
    }
}