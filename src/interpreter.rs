use crate::ast::*;
use crate::cards::{standard_deck, Card, Player};



pub struct Game {
    name: Option<String>,
    deck: Vec<Card>,
    players: Vec<Player>
}

impl Game {
    pub fn new(ast: Vec<Statement>) -> Game {
        let mut deck = standard_deck();
        let mut name = None;
        let mut players = vec!();

        for statement in ast.iter() {
            match statement {
                Statement::Declaration(
                    Declaration{
                        key: GlobalKey::Name,
                        value: Expression::Symbol(v)
                    }
                ) => {
                    name = Some(v.to_string());
                },
                _ => ()
            }
        }

        Game{ deck, name, players }
    }

    pub fn show(&self, key: &str) -> String {
        match key {
            "deck" => self.display_deck(),
            "name" => self.display_name(),
            _ => format!("Unknown item: {}", key)
        }
    }

    fn display_deck(&self) -> String {
        self.deck.iter().map(|x|x.to_string()).collect::<Vec<String>>().join(", ")
    }

    fn display_name(&self) -> String {
        match &self.name {
            Some(name) => name.to_string(),
            None => "Name not initalised!".to_string() // TODO - Error? Default? 
         }
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

    #[test]
    fn it_can_display_a_name() {
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Name,
                    value: Expression::Symbol("turns".to_string())
                }
            )
        );

        let game = Game::new(ast);
        let name = game.show("name");

        assert_eq!(name, "turns".to_string());
    }
}