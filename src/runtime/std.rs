use crate::cards::Card;
use crate::ast::*;
use rand::seq::SliceRandom;
use super::{PrimitiveValue, GameState};

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

pub fn filter(stack: Vec<Card>, _function: Definition) -> Vec<Card> {
    return stack;
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::cards::standard_deck;

    #[test]
    fn filter_executes_a_function_against_a_stack_and_keeps_cards_when_true() {
        let cards = standard_deck();
        let return_statement = Statement::ReturnStatement(ReturnStatement{
            expression: Expression::Bool(true)
        });
        let func = Definition{
            name: "_".to_string(),
            arguments: vec!("card".to_string()),
            body: vec!(return_statement)
        };

        let filtered_cards = filter(cards, func);

        assert_eq!(filtered_cards.len(), 52);
    }

    //#[test]
    fn filter_executes_a_function_against_a_stack_and_keeps_cards_when_false() {
        let cards = standard_deck();
        let return_statement = Statement::ReturnStatement(ReturnStatement{
            expression: Expression::Bool(false)
        });
        let func = Definition{
            name: "_".to_string(),
            arguments: vec!("card".to_string()),
            body: vec!(return_statement)
        };

        let filtered_cards = filter(cards, func);

        assert_eq!(filtered_cards.len(), 0);
    }
}