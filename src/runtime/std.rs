use crate::cards::Card;
use crate::interpreter::{GameState, PrimitiveValue};
use rand::seq::SliceRandom;

pub fn shuffle(stack: &mut Vec<Card>) {
    let mut rng = rand::thread_rng();
    stack.shuffle(&mut rng);
}

pub fn end(status: &mut GameState) {
    *status = GameState::GameOver;
}

pub fn winner(winners: &mut Vec<f64>, player: f64) {
    winners.push(player);
}

pub fn count(stack: PrimitiveValue) -> usize {
    match stack {
        PrimitiveValue::Stack(v) => v.len(),
        _ => 0
    }
}