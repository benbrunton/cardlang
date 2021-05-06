use crate::ast::*;
use crate::cards::{standard_deck, Card, Player};
use std::fmt::Display;


#[derive(Clone)]
pub struct Game {
    name: Option<String>,
    deck: Vec<Card>,
    players: Vec<Player>,
    setup: Vec<Statement>,
    ast: Vec<Statement>
}

pub enum TransferTarget {
    Stack(Stack),
    StackList(Vec<Stack>)
}

impl TransferTarget {
    pub fn count(&self) -> usize {
        match self {
            TransferTarget::Stack(s) => s.len(),
            TransferTarget::StackList(s) => s.len()
        }
    }
}

type Stack = Vec<Card>;

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

        let mut g = Game{ deck, name, players, setup, ast };
        g.initialise_declarations();

        g
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

        let mut count = match transfer.count {
            None => 1,
            Some(TransferCount::End) => from.as_ref().unwrap().count()
        };

        // multiply by number of target stacks
        count *= match &to {
            Some(TransferTarget::Stack(_)) => 1,
            Some(TransferTarget::StackList(s)) => s.len(),
            _ => 0
        };

        let mut transfer_index = 0;

        while count > 0 {

            let card_result = match from {
                Some(TransferTarget::Stack(ref mut s)) => s.pop(),
                _ => None
            };

            // todo - error?
            if card_result.is_none() {
                break;
            }

            if to.is_none() {
                return;
            }

            let card = card_result.expect("unable to get card");

            match to {
                Some(TransferTarget::StackList(ref mut s)) => {
                    s[transfer_index].push(card);
                    if transfer_index >= s.len() - 1 {
                        transfer_index = 0;
                    } else {
                        transfer_index += 1
                    }
                },
                _ => ()
            }

            count -= 1;
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
}