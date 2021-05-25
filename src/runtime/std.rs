use crate::cards::Card;
use crate::interpreter::{GameState, PrimitiveValue};
use crate::ast::Definition;
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

pub fn filter(stack: Vec<Card>, function: Definition) {
    unimplemented!()
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::cards::standard_deck;

    //#[test]
    fn filter_executes_a_function_against_a_stack_and_keeps_cards_when_true() {
        let cards = standard_deck();
        let func = Definition{
            name: "_".to_string(),
            arguments: vec!(),
            body: vec!()
        };

        let filtered_cards = filter(cards, func);
    }
}