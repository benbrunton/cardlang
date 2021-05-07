use crate::ast::*;
use crate::cards::{standard_deck, Card, Player};
use std::fmt::Display;
use crate::runtime::{transfer::{transfer, TransferTarget}, std::inbuilt};

#[derive(Clone)]
pub struct Game {
    name: Option<String>,
    deck: Vec<Card>,
    players: Vec<Player>,
    setup: Vec<Statement>,
    ast: Vec<Statement>
}

impl Game {
    pub fn new(ast: Vec<Statement>) -> Game {
        let deck = vec!();
        let name = None;
        let players = vec!();
        let mut setup = vec!();

        for statement in ast.iter() {
            match statement {
                Statement::Definition(
                    Definition{
                        name,
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
        let mut game = Game{ deck, name, players, setup, ast };
        game.initialise_declarations();
        game
    }

    fn initialise_declarations(&mut self) {
        self.deck = standard_deck();
        for statement in self.ast.iter() {
            match statement {
                Statement::Declaration(
                    Declaration{
                        key: GlobalKey::Name,
                        value: Expression::Symbol(v)
                    }
                ) => {
                    self.name = Some(v.to_string());
                },
                Statement::Declaration(
                    Declaration{
                        key: GlobalKey::Players,
                        value: Expression::Number(n)
                    }
                ) => {
                    self.players = Self::generate_players(*n as i32);
                },
                _ => ()
            }
        }
    }

    pub fn show(&self, key: &str) -> String {
        match key {
            "deck" => Self::display_list(&self.deck),
            "name" => self.display_name(),
            "players" => Self::display_list(&self.players),
            _ => self.check_exploded_show(key)
        }
    }

    fn check_exploded_show(&self, key: &str) -> String {
        let instructions: Vec<&str> = key.split(" ").collect();
        match instructions[0] {
            "player" => self.handle_show_player(instructions),
            _ => format!("Unknown item: {}", key)
        }
    }

    fn handle_show_player(&self, args: Vec<&str>) -> String {
        let player_num = args[1].parse::<usize>().unwrap_or(1) - 1;
        Self::display_list(&self.players[player_num].get_hand())
    }

    pub fn start(&mut self) {
        self.initialise_declarations();
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
                Statement::FunctionCall(f) => self.handle_function_call(f),
                _ => ()
            }
        })
    }

    // todo - handle failures
    fn handle_transfer(&mut self, t: &Transfer) {
        let from = self.get_stack(&t.from);
        let to = self.get_stack(&t.to);

        let transfer_result = transfer(to, from, t.count.as_ref());

        let (new_from, new_to) = match transfer_result {
            Some((a, b)) => (a, b),
            _ => return
        };

        self.set_stack(&t.from, new_from);
        self.set_stack(&t.to, new_to);
    }

    fn handle_function_call(&mut self, f: &FunctionCall) {
        /*
            grab function
            pass arguments num, stack, stack o stacks
        */

        let resolved_args = vec!(&mut self.deck);
        let _func_result = inbuilt(&f.name, resolved_args);
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
            "deck" => self.deck = stack.get_stack(0),
            "players" => self.players.iter_mut().enumerate().for_each(|(n, p)| {
                let new_hand = stack.get_stack(n);
                p.set_hand(new_hand)
            }),
            _ => ()
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

        assert_eq!(players, "player 1 (cards: 0), player 2 (cards: 0), player 3 (cards: 0)".to_string());
    }

    #[test]
    fn it_can_display_a_single_player() {
        let ast = vec!(
            Statement::Declaration (
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(1.0)
                }
            )
        );

        let game = Game::new(ast);
        let players = game.show("players");

        assert_eq!(players, "player 1 (cards: 0)".to_string());
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
        let modifier = None;
        let count = None;
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

        assert_eq!(split_deck.len(), 49);
    }

    #[test]
    fn second_start_restarts() {
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
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec!(transfer_statement);
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();
        game.start();

        let deck = game.show("deck");
        let split_deck: Vec<&str> = deck.split(",").collect();

        assert_eq!(split_deck.len(), 49);
    }

    #[test]
    fn it_deals_to_the_end_with_the_count_modifier() {
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
        let count = Some(TransferCount::End);
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
        assert_eq!(&deck, "");
    }

    #[test]
    fn it_can_show_player_hand(){
        let mut ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(1.0)
                }
            )
        );
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None; //Some(TransferModifier::Alternate);
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec!(transfer_statement);
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();

        let hand = game.show("player 1 hand");
        assert_eq!(&hand, "king diamonds");
    }

    #[test]
    fn it_can_show_multiple_player_hand(){
        let mut ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(2.0)
                }
            )
        );
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None; //Some(TransferModifier::Alternate);
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec!(transfer_statement);
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();

        let show_players = game.show("players");
        assert_eq!(&show_players, "player 1 (cards: 1), player 2 (cards: 1)");

        let hand = game.show("player 2 hand");
        assert_eq!(&hand, "queen diamonds");
    }

    #[test]
    fn it_can_access_built_in_functions() {
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "shuffle".to_string(),
                    arguments: vec!(Expression::Symbol("deck".to_string()))
                }
            )
        );

        let name = "setup".to_owned();
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let usual_order = Game::display_list(&standard_deck());
        let deck = game.show("deck");
        let split_deck: Vec<&str> = deck.split(",").collect();

        assert_ne!(deck, usual_order);
    }
}