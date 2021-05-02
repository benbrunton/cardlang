use crate::ast::*;
use crate::cards::standard_deck;

pub struct Game {

}

impl Game {
    pub fn new(ast: Vec<Statement>) -> Game {
        Game{}
    }

    pub fn show(&self, key: &str) -> String {
        let deck = standard_deck();
        deck.iter().map(|x|x.to_string()).collect::<Vec<String>>().join(", ")
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn it_can_display_a_deck() {
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Deck,
                    value: Expression::Symbol("StandardDeck".to_string())
                }
            )
        );

        let game = Game::new(ast);
        let deck = game.show("deck");
        let split_deck: Vec<&str> = deck.split(",").collect();

        assert_eq!(split_deck[0], "ace spades");
        assert_eq!(split_deck.len(), 52);
    }
}