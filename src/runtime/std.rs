use crate::cards::Card;
use crate::interpreter::GameState;
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