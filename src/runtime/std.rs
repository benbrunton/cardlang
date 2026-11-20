use crate::cards::Card;
use rand::seq::SliceRandom;
use super::{PrimitiveValue, GameState};

pub fn shuffle(stack: &mut [Card]) {
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

// true when every card in `cards` is in `stack` (each card can only match once)
pub fn cards_in_stack(cards: &[Card], stack: &[Card]) -> bool {
    let mut remaining = stack.to_vec();
    for card in cards {
        match remaining.iter().position(|c| c == card) {
            Some(i) => { remaining.remove(i); },
            None => return false
        }
    }
    true
}
