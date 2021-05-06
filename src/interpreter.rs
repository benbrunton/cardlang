use crate::ast::*;
use crate::cards::{standard_deck, Card, Player};
use std::fmt::Display;


#[derive(Clone)]
pub struct Game {
    name: Option<String>,
    deck: Vec<Card>,
    players: Vec<Player>,
    setup: Vec<Statement>
}

pub enum TransferTarget {
    Stack(Stack),
    StackList(Vec<Stack>)
}

type Stack = Vec<Card>;

impl Game {
    pub fn new(ast: Vec<Statement>) -> Game {
        let deck = standard_deck();
        let mut name = None;
        let mut players = vec!();
        let mut setup = vec!();

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
                Statement::Declaration(
                    Declaration{
                        key: GlobalKey::Players,
                        value: Expression::Number(n)
                    }
                ) => {
                    players = Self::generate_players(*n as i32);
                },
                Statement::Definition(
                    Definition{
                        name: name,
                        body: b
                    }
                ) => {
                    match name.as_str() {
                        "setup" => {setup = b.to_vec();},
                        _ => ()
                    }
                    
                },
                _ => ()
            }
        }

        Game{ deck, name, players, setup }
    }

    pub fn show(&self, key: &str) -> String {
        match key {
            "deck" => Self::display_list(&self.deck),
            "name" => self.display_name(),
            "players" => Self::display_list(&self.players),
            _ => format!("Unknown item: {}", key)
        }
    }

    pub fn start(&mut self) {
        self.handle_statements(&self.setup.clone());
    }

    fn display_name(&self) -> String {
        match &self.name {
            Some(name) => name.to_string(),
            None => "Name not initalised!".to_string() // TODO - Error? Default? 
         }
    }
    
    fn handle_statements(&mut self, statements: &Vec<Statement>){
        statements.iter().for_each(|statement|{
            match statement {
                Statement::Transfer(t) => self.handle_transfer(t),
                _ => ()
            }
        })
    }

    // todo - handle failures
    fn handle_transfer(&mut self, transfer: &Transfer) {
        /*
            Transfer {
                pub from: String,
                pub to: String,
                pub modifier: Option<TransferModifier>,
                pub count: Option<TransferCount>
            }

            find from stack
            find to stack (if list then stack list)

            move card from from to to

            set from to new from
            set to to new to

        let mut count = 0;
        loop {
            match transfer.count
        }
        */

        let mut from = self.get_stack(&transfer.from);
        let mut to = self.get_stack(&transfer.to);

        let card_result = match from {
            Some(TransferTarget::Stack(ref mut s)) => s.pop(),
            _ => None
        };

        // todo - error?
        if card_result.is_none() {
            return;
        }

        if to.is_none() {
            return;
        }

        let card = card_result.expect("unable to get card");

        match to {
            Some(TransferTarget::StackList(ref mut s)) => s[0].push(card),
            _ => ()
        }

        self.set_stack(&transfer.from, from.expect("unable to find stack (from)"));
        self.set_stack(&transfer.to, to.expect("unable to find stack (to)"));
    }

    fn get_stack(&self, stack_key: &str) -> Option<TransferTarget> {
        match stack_key {
            "deck" => Some(TransferTarget::Stack(self.deck.clone())),
            "players" => Some(TransferTarget::StackList(self.players.iter().map(|p| p.get_hand()).collect())),
            _ => None
        }
    }

    fn set_stack(&mut self, stack_key: &str, stack: TransferTarget) {
        match stack_key {
            "deck" => self.deck = Self::get_stack_from_transfer(&stack, 0),
            "players" => self.players.iter_mut().enumerate().for_each(|(n, p)| {
                let new_hand = Self::get_stack_from_transfer(&stack, n);
                p.set_hand(new_hand)
            }),
            _ => ()
        }
    }

    fn get_stack_from_transfer(stack: &TransferTarget, n: usize) -> Stack {
        match stack {
            TransferTarget::Stack(s) => s.clone(),
            TransferTarget::StackList(s) => s[n].clone()
        }
    }

    fn display_list<D: Display>(list: &Vec<D>) -> String {
        list.iter().map(|x|x.to_string()).collect::<Vec<String>>().join(", ")
    }

    fn generate_players(n: i32) -> Vec<Player>{
        let mut players = vec!();
        for i in 0..n {
            players.push(
                Player::new(i + 1)
            );
        }
        players
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

    #[test]
    fn it_can_display_players() {
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(3.0)
                }
            )
        );

        let game = Game::new(ast);
        let players = game.show("players");

        assert_eq!(players, "player 1, player 2, player 3".to_string());
    }

    #[test]
    fn it_can_display_a_single_player() {
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(1.0)
                }
            )
        );

        let game = Game::new(ast);
        let players = game.show("players");

        assert_eq!(players, "player 1".to_string());
    }

    #[test]
    fn it_can_start_a_game() {
        let mut ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(3.0)
                }
            )
        );
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None; //Some(TransferModifier::Alternate);
        let count = None; //Some(TransferCount::End);
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec!(transfer_statement);
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();

        let deck = game.show("deck");
        let split_deck: Vec<&str> = deck.split(",").collect();

        assert_eq!(split_deck.len(), 51);
    }
}