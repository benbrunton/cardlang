use crate::ast::*;
use std::fmt::Display;
use std::collections::HashMap;
use crate::runtime::{
    Runtime,
    InitialValues,
    Callbacks
};

#[derive(Clone, Debug)]
pub struct Game {
    name: Option<String>,
    ast: Vec<Statement>,
    runtime: Runtime,
    initial_values: InitialValues,
    callbacks: Callbacks
}

impl Game {
    pub fn new(ast: Vec<Statement>) -> Game {
        let mut name = None;

        let mut initial_values = InitialValues{ 
            players: 1,
            player_stacks: vec!(),
            card_values: vec!(),
            card_stacks: vec!(),
            current_player: 1 
        };

        let mut callbacks = Callbacks {
            player_move: None,
            setup: None,
            definitions: HashMap::new()
        };

        for statement in ast.iter() {
            match statement {
                Statement::Definition(
                    d
                ) => {
                    callbacks.definitions.insert(d.name.clone(), d.clone());
                    match d.name.as_str() {
                        "setup" => callbacks.setup = Some(d.clone()),
                        "player_move" => callbacks.player_move = Some(d.clone()),
                        _ => ()
                    }
                },
                Statement::Declaration(Declaration{
                    key: GlobalKey::Name,
                    value: Expression::Symbol(v)
                }) => {
                    name = Some(v.to_string());
                },
                Statement::Declaration(Declaration{
                    key: GlobalKey::Players,
                    value: Expression::Number(n)
                }) => {
                    initial_values.players = *n as u32;
                },
                Statement::Declaration(Declaration{
                    key: GlobalKey::CurrentPlayer,
                    value: Expression::Number(n)
                }) => {
                    initial_values.current_player = *n as usize;
                },
                Statement::Declaration(Declaration{
                    key: GlobalKey::Stack,
                    value: Expression::Symbol(s)
                }) => {
                    initial_values.card_stacks.push(s.to_string());
                },
                Statement::ValueTable(table) => {
                    initial_values.card_values.extend(table.iter().cloned());
                },
                Statement::Declaration(Declaration{
                    key: GlobalKey::PlayerStack,
                    value: Expression::Symbol(s)
                }) => {
                    initial_values.player_stacks.push(s.to_string());
                },
                _ => ()
            }

        }

        let runtime = Runtime::new(initial_values.clone(), callbacks.clone());

        Game {
            name,
            ast,
            runtime,
            initial_values: initial_values.clone(),
            callbacks: callbacks.clone()
        }
    }

    pub fn show(&self, key: &str) -> String {
        match key {
            "deck" => Self::display_list(&self.runtime.get_deck()),
            "name" => self.display_name(),
            "players" => Self::display_list(&self.runtime.get_players()),
            "game" => {
                let winner_list = self.runtime.get_winners();
                let winners = if winner_list.len() > 0 {
                    let w = Self::display_list(&winner_list);
                    format!("\nwinners: {}", w)
                } else {
                    "".to_string()
                };
                let status = self.runtime.get_status();
                format!("{}{}", status, winners)
            },
            "current_player" => {
                format!("{}", self.runtime.get_current_player())
            },
            _ => self.check_exploded_show(key)
        }
    }

    pub fn start(&mut self) {
        self.runtime = Runtime::new(self.initial_values.clone(), self.callbacks.clone());
        self.runtime.setup();
    }

    pub fn player_move(&mut self, player: usize) -> Result<(), String> {
        self.runtime.player_move(player)
    }

    fn check_exploded_show(&self, key: &str) -> String {
        let instructions: Vec<&str> = key.split(" ").collect();
        match instructions[0] {
            "player" => self.handle_show_player(instructions),
            key => self.find_custom_item(key)
        }
    }

    fn handle_show_player(&self, args: Vec<&str>) -> String {
        let player_num = match args.get(1).map(|a| a.parse::<usize>()) {
            Some(Ok(n)) if n >= 1 => n,
            _ => return "usage: show player <number>".to_string()
        };
        let stack_name = args.get(2).copied().unwrap_or("hand");
        match self.runtime.get_player(player_num - 1) {
            Some(player) => match player.get_stack(stack_name) {
                Some(cards) => Self::display_list(&cards),
                None => format!("player {} has no stack '{}'", player_num, stack_name)
            },
            None => format!("there is no player {}", player_num)
        }
    }

    fn display_name(&self) -> String {
        match &self.name {
            Some(name) => name.to_string(),
            None => "Name not initalised!".to_string() // TODO - Error? Default? 
         }
    }

    fn display_list<D: Display>(list: &Vec<D>) -> String {
        list.iter().map(|x|x.to_string()).collect::<Vec<String>>().join(", ")
    }

    fn find_custom_item(&self, key: &str) -> String {
        match self.runtime.find_custom_item(key) {
            Some(v) => Self::display_list(&v),
            _ => format!("{} not found", key)
        }
    }

}


/*


######################################
//////////////////////////////////////
///////////// TESTS //////////////////
//////////////////////////////////////
######################################



*/

#[cfg(test)]
mod test{
    use super::*;
    use crate::cards::standard_deck;

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
        let definition = Definition{ arguments: vec!(), name, body };
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
        let definition = Definition{ arguments: vec!(), name, body };
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
        let definition = Definition{ arguments: vec!(), name, body };
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
        let definition = Definition{ arguments: vec!(), name, body };
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
        let definition = Definition{ arguments: vec!(), name, body };
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
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let usual_order = Game::display_list(&standard_deck());
        let deck = game.show("deck");

        assert_ne!(deck, usual_order);
    }

    #[test]
    fn it_can_make_a_move() {
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "shuffle".to_string(),
                    arguments: vec!(Expression::Symbol("deck".to_string()))
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);

        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();
        game.player_move(1).unwrap();

        let usual_order = Game::display_list(&standard_deck());
        let deck = game.show("deck");

        assert_ne!(deck, usual_order);
    }

    #[test]
    fn it_passes_the_player_to_the_move() {
        let players = Statement::Declaration(
            Declaration {
                key: GlobalKey::Players,
                value: Expression::Number(3.0)
            }
        );

        let body = vec!(
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "player:hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!("player".to_string()), name, body };
        let statement = Statement::Definition(definition);

        let ast = vec!(
            players,
            statement
        );

        let mut game = Game::new(ast);
        game.start();
        game.player_move(1).unwrap();

        let player_hand = game.show("player 1 hand");

        assert_eq!(player_hand, "king diamonds".to_string());
    }

    #[test]
    fn it_passes_the_player_num_to_the_move() {
        let players = Statement::Declaration(
            Declaration {
                key: GlobalKey::Players,
                value: Expression::Number(3.0)
            }
        );

        let body = vec!(
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "player:hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!("player".to_string()), name, body };
        let statement = Statement::Definition(definition);

        let ast = vec!(
            players,
            statement
        );

        let mut game = Game::new(ast);
        game.start();
        game.player_move(2).unwrap();

        let player1_hand = game.show("player 1 hand");
        let player2_hand = game.show("player 2 hand");

        assert_eq!(&player1_hand, "");
        assert_eq!(&player2_hand, "king diamonds");
    }

    #[test]
    fn it_can_handle_custom_stacks() {
        let mut ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(1.0)
                }
            ),
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Stack,
                    value: Expression::Symbol("middle".to_string())
                }
            )
        );
        let from = "deck".to_owned();
        let to = "middle".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec!(transfer_statement);
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();

        let middle = game.show("middle");

        assert_eq!(&middle, "king diamonds");
    }

    #[test]
    fn it_can_show_info_about_the_game() {
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Deck,
                    value: Expression::Symbol("StandardDeck".to_string())
                }
            )
        );

        let game = Game::new(ast);
        let display = game.show("game");

        assert_eq!(display, "pending"); 
    }

    #[test]
    fn it_can_end_a_game() {
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over");
    }

    #[test]
    fn it_doesnt_move_when_game_hasnt_started() {
        let players = Statement::Declaration(
            Declaration {
                key: GlobalKey::Players,
                value: Expression::Number(3.0)
            }
        );

        let body = vec!(
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "player hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);

        let ast = vec!(
            players,
            statement
        );

        let mut game = Game::new(ast);
        assert!(game.player_move(1).is_err());

        let player_hand = game.show("player 1 hand");

        assert_eq!(player_hand, "".to_string());
    }

    #[test]
    fn it_doesnt_move_when_game_over() {
        let players = Statement::Declaration(
            Declaration {
                key: GlobalKey::Players,
                value: Expression::Number(3.0)
            }
        );

        let body = vec!(
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "player hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);

        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_owned(),
                    arguments: vec!()
                }
            )
        );
        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let setup = Statement::Definition(definition);

        let ast = vec!(
            players,
            statement,
            setup
        );

        let mut game = Game::new(ast);
        game.start();
        assert!(game.player_move(1).is_err());

        let player_hand = game.show("player 1 hand");

        assert_eq!(player_hand, "".to_string());
    }

    #[test]
    fn it_can_apply_a_winner() {
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "winner".to_string(),
                    arguments: vec!(Expression::Number(1.0))
                }
            )
        );

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "active\nwinners: 1");
    }

    #[test]
    fn it_can_apply_a_winner_by_id() {
        let declaration = Statement::Declaration(
            Declaration {
                key: GlobalKey::Players,
                value: Expression::Number(1.0)
            }
        );
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "winner".to_string(),
                    arguments: vec!(Expression::Symbol("player:id".to_string()))
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!("player".to_string()), name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(declaration, statement);

        let mut game = Game::new(ast);
        game.start();
        game.player_move(1).unwrap();

        let display = game.show("game");

        assert_eq!(display, "active\nwinners: 1");
    }

    #[test]
    fn it_can_show_a_winner_after_game_over() {
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "winner".to_string(),
                    arguments: vec!(Expression::Number(1.0))
                }
            ),
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over\nwinners: 1");
    }

    #[test]
    fn it_executes_if_statement_when_expression_is_true() {
        let if_body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let if_statement = IfStatement{ else_body: vec!(),
            expression: Expression::Bool(true),
            body: if_body
        };

        let body = vec!(
            Statement::IfStatement(if_statement)
        );

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over");
    }

    #[test]
    fn it_doesnt_execute_if_statement_when_expression_is_false() {
        let if_body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let if_statement = IfStatement{ else_body: vec!(),
            expression: Expression::Bool(false),
            body: if_body
        };

        let body = vec!(
            Statement::IfStatement(if_statement)
        );

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "active");
    }

    #[test]
    fn it_executes_if_statement_when_expression_is_true_comparison() {
        let if_body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let comparison = Comparison{
            left: Expression::Number(1.0),
            right: Expression::Number(1.0),
            negative: false
        };

        let if_statement = IfStatement{ else_body: vec!(),
            expression: Expression::Comparison(Box::new(comparison)),
            body: if_body
        };

        let body = vec!(
            Statement::IfStatement(if_statement)
        );

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec!() };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over");
    }

    #[test]
    fn it_can_compare_based_on_function_calls() {
        let mut ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(2.0)
                }
            )
        );
        let if_body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let count_call = FunctionCall {
            name: "count".to_string(),
            arguments: vec!(
                Expression::Symbol("player:hand".to_string())
            )
        };

        let comparison = Comparison{
            left: Expression::FunctionCall(count_call),
            right: Expression::Number(0.0),
            negative: false
        };

        let if_statement = IfStatement{ else_body: vec!(),
            expression: Expression::Comparison(Box::new(comparison)),
            body: if_body
        };

        let body = vec!(
            Statement::IfStatement(if_statement)
        );

        let name = "player_move".to_owned();
        let definition = Definition{ name, body, arguments: vec!() };
        let statement = Statement::Definition(definition);
        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();
        game.player_move(1).unwrap();

        let display = game.show("game");

        assert_eq!(display, "game over");
    }

    #[test]
    fn it_can_compare_based_on_function_calls_with_cards() {
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
        let count = Some(TransferCount::End);
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec!(transfer_statement);
        let definition = Definition{ name, body, arguments: vec!() };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let if_body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let count_call = FunctionCall {
            name: "count".to_string(),
            arguments: vec!(
                Expression::Symbol("player:hand".to_string())
            )
        };

        let comparison = Comparison{
            left: Expression::FunctionCall(count_call),
            right: Expression::Number(26.0),
            negative: false
        };

        let if_statement = IfStatement{ else_body: vec!(),
            expression: Expression::Comparison(Box::new(comparison)),
            body: if_body
        };

        let body = vec!(
            Statement::IfStatement(if_statement)
        );

        let name = "player_move".to_owned();
        let definition = Definition{ name, body, arguments: vec!("player".to_string()) };
        let statement = Statement::Definition(definition);
        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();
        game.player_move(1).unwrap();

        let display = game.show("game");

        assert_eq!(display, "game over");
    }

    #[test]
    fn check_stops_a_function_executing_when_expression_is_false() {
        let body = vec!(
            Statement::CheckStatement(CheckStatement{
                expression: Expression::Bool(false)
            }),
            Statement::FunctionCall(
                FunctionCall{
                    name: "winner".to_string(),
                    arguments: vec!(Expression::Number(1.0))
                }
            ),
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec!() };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "active");
    }

    #[test]
    fn check_passes_through_when_expression_is_true() {
        let body = vec!(
            Statement::CheckStatement(CheckStatement{
                expression: Expression::Bool(true)
            }),
            Statement::FunctionCall(
                FunctionCall{
                    name: "winner".to_string(),
                    arguments: vec!(Expression::Number(1.0))
                }
            ),
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec!() };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over\nwinners: 1");
    }

    #[test]
    fn it_shows_current_player() {
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::CurrentPlayer,
                    value: Expression::Number(1.0)
                }
            )
        );

        let game = Game::new(ast);
        let current_player = game.show("current_player");

        assert_eq!(current_player, "1");
    }

    #[test]
    fn it_shows_current_player_as_set() {
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::CurrentPlayer,
                    value: Expression::Number(2.0)
                }
            )
        );

        let game = Game::new(ast);
        let current_player = game.show("current_player");

        assert_eq!(current_player, "2");
    }

    #[test]
    fn it_can_rotate_current_player() {
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "next_player".to_string(),
                    arguments: vec!()
                }
            )
        );

        let name = "setup".to_owned();
        let definition = Definition{ name, body,  arguments: vec!(), };
        let statement = Statement::Definition(definition);
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(3.0)
                },
            ),
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::CurrentPlayer,
                    value: Expression::Number(1.0)
                }
            ),
            statement
        );

        let mut game = Game::new(ast);
        game.start();

        let current_player = game.show("current_player");
        assert_eq!(current_player, "2");
    }

    #[test]
    fn it_can_rotate_current_player_back_to_first() {
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "next_player".to_string(),
                    arguments: vec!()
                }
            )
        );

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec!() };
        let statement = Statement::Definition(definition);
        let ast = vec!(
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(2.0)
                },
            ),
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::CurrentPlayer,
                    value: Expression::Number(2.0)
                }
            ),
            statement
        );

        let mut game = Game::new(ast);
        game.start();

        let current_player = game.show("current_player");
        assert_eq!(current_player, "1");
    }

    #[test]
    fn it_executes_if_statement_when_expression_is_true_and_true() {
        let if_body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        );

        let and = And{
            left: Expression::Bool(true),
            right: Expression::Bool(true)
        };

        let if_statement = IfStatement{ else_body: vec!(),
            expression: Expression::And(Box::new(and)),
            body: if_body
        };

        let body = vec!(
            Statement::IfStatement(if_statement)
        );

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec!() };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over");
    }

    #[test]
    fn it_passes_the_player_to_the_move_with_the_specified_argument_label() {
        let players = Statement::Declaration(
            Declaration {
                key: GlobalKey::Players,
                value: Expression::Number(3.0)
            }
        );

        let body = vec!(
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "pl:hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!("pl".to_string()), name, body };
        let statement = Statement::Definition(definition);

        let ast = vec!(
            players,
            statement
        );

        let mut game = Game::new(ast);
        game.start();
        game.player_move(1).unwrap();

        let player_hand = game.show("player 1 hand");

        assert_eq!(player_hand, "king diamonds".to_string());
    }

    fn game_from_source(source: &str) -> Game {
        let tokens = crate::lex::lexer(source).expect("lex error");
        let ast = crate::parse::parse(&tokens).expect("parse error");
        Game::new(ast)
    }

    #[test]
    fn it_can_call_a_user_defined_function_with_arguments() {
        let mut game = game_from_source("
players 2

def finish(id){
    winner(id)
    end()
}

def setup(){
    finish(2)
}
");
        game.start();

        assert_eq!(game.show("game"), "game over\nwinners: 2");
    }

    #[test]
    fn it_can_use_the_return_value_of_a_user_defined_function() {
        let mut game = game_from_source("
players 2

def has_no_cards(p){
    return(count(p:hand) is 0)
}

def player_move(player){
    check(has_no_cards(player) is true)
    end()
}
");
        game.start();
        game.player_move(1).unwrap();

        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn a_failing_check_in_a_user_defined_function_returns_false() {
        let mut game = game_from_source("
players 2

def has_no_cards(p){
    return(count(p:hand) is 0)
}

def setup(){
    deck > players end
}

def player_move(player){
    check(has_no_cards(player) is true)
    end()
}
");
        game.start();
        game.player_move(1).unwrap();

        assert_eq!(game.show("game"), "active");
    }

    #[test]
    fn user_defined_functions_can_transfer_from_a_player_passed_in() {
        let mut game = game_from_source("
players 2
stack middle

def discard(p){
    p:hand > middle
}

def setup(){
    deck > players
}

def player_move(player){
    discard(player)
}
");
        game.start();
        game.player_move(1).unwrap();

        assert_eq!(game.show("player 1 hand"), "");
        assert_eq!(game.show("middle"), "king diamonds");
    }

    #[test]
    fn arguments_are_scoped_to_the_function_call() {
        let mut game = game_from_source("
players 2

def inner(id){
    winner(id)
}

def outer(id){
    inner(1)
    winner(id)
}

def setup(){
    outer(2)
}
");
        game.start();

        assert_eq!(game.show("game"), "active\nwinners: 1, 2");
    }

    #[test]
    fn filter_can_be_called_with_a_user_defined_function() {
        let mut game = game_from_source("
def is_ace(card){
    return(card:rank is Ace)
}

def setup(){
    if(count(filter(deck, is_ace)) is 4){
        end()
    }
}
");
        game.start();

        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn filter_can_use_negative_comparisons() {
        let mut game = game_from_source("
def not_ace(card){
    return(card:rank is not Ace)
}

def setup(){
    if(count(filter(deck, not_ace)) is 48){
        end()
    }
}
");
        game.start();

        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn custom_stacks_can_be_counted() {
        let mut game = game_from_source("
stack middle

def setup(){
    deck > middle
    if(count(middle) is 1){
        end()
    }
}
");
        game.start();

        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn infinite_recursion_is_stopped() {
        let mut game = game_from_source("
def forever(){
    forever()
}

def setup(){
    forever()
    end()
}
");
        game.start();

        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn player_hand_reflects_transfers_made_during_the_move() {
        let mut game = game_from_source("
players 2
stack middle

def setup(){
    deck > players
}

def player_move(player){
    player:hand > middle
    if(count(player:hand) is 0){
        end()
    }
}
");
        game.start();
        game.player_move(1).unwrap();

        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn built_ins_called_without_arguments_do_not_panic() {
        let mut game = game_from_source("
players 2

def setup(){
    winner()
    if(count() is 0){
        end()
    }
}
");
        game.start();

        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn transferring_everything_from_an_unknown_stack_does_nothing() {
        let mut game = game_from_source("
players 2

def setup(){
    nowhere > deck end
}
");
        game.start();

        assert_eq!(game.show("game"), "active");
    }

    #[test]
    fn moving_without_a_player_move_function_is_an_error() {
        let mut game = game_from_source("
players 2
");
        game.start();

        assert!(game.player_move(1).is_err());
    }

    #[test]
    fn moving_for_a_player_that_does_not_exist_is_an_error() {
        let mut game = game_from_source("
players 2

def player_move(player){
    end()
}
");
        game.start();

        assert!(game.player_move(0).is_err());
        assert!(game.player_move(3).is_err());
        assert_eq!(game.show("game"), "active");
        assert!(game.player_move(2).is_ok());
    }

    #[test]
    fn showing_a_player_that_does_not_exist_gives_a_message() {
        let game = game_from_source("
players 2
");

        assert_eq!(game.show("player"), "usage: show player <number>");
        assert_eq!(game.show("player x"), "usage: show player <number>");
        assert_eq!(game.show("player 0"), "usage: show player <number>");
        assert_eq!(game.show("player 3"), "there is no player 3");
        assert_eq!(game.show("player 2"), "");
    }

    fn winners_of(source: &str) -> String {
        let mut game = game_from_source(source);
        game.start();
        game.show("game")
    }

    #[test]
    fn return_inside_an_if_returns_from_the_function() {
        let result = winners_of("
players 2

def pick(a, b){
    if(a is 1){
        return(10)
    }
    return(b)
}

def setup(){
    winner(pick(1, 20))
    winner(pick(2, 30))
}
");
        assert_eq!(result, "active\nwinners: 10, 30");
    }

    #[test]
    fn check_inside_an_if_ends_the_function() {
        let result = winners_of("
players 2

def setup(){
    if(true){
        check(false)
    }
    end()
}
");
        assert_eq!(result, "active");
    }

    #[test]
    fn and_binds_looser_than_comparisons() {
        let result = winners_of("
players 2

def setup(){
    if(1 is 1 & 2 is 2){
        winner(1)
    }
    if(1 is 1 & 2 is 3){
        winner(2)
    }
    if(1 is not 2 & 3 is not 4){
        winner(3)
    }
}
");
        assert_eq!(result, "active\nwinners: 1, 3");
    }

    #[test]
    fn or_combines_conditions_and_binds_looser_than_and() {
        let result = winners_of("
players 2

def setup(){
    if(1 is 2 or 3 is 3){
        winner(1)
    }
    if(1 is 2 or 3 is 4){
        winner(2)
    }
    if(true or false & false){
        winner(3)
    }
    if((true or false) & false){
        winner(4)
    }
}
");
        assert_eq!(result, "active\nwinners: 1, 3");
    }

    #[test]
    fn booleans_and_function_calls_can_be_used_directly_as_conditions() {
        let result = winners_of("
players 2

def yes(){
    return(true)
}

def no(){
    return(false)
}

def setup(){
    if(yes()){
        winner(1)
    }
    if(no()){
        winner(2)
    }
    if(true & yes()){
        winner(3)
    }
    if(no() or yes()){
        winner(4)
    }
    check(yes())
    winner(5)
}
");
        assert_eq!(result, "active\nwinners: 1, 3, 4, 5");
    }

    #[test]
    fn non_boolean_values_are_not_true() {
        let result = winners_of("
players 2

def one(){
    return(1)
}

def setup(){
    if(one()){
        winner(1)
    }
}
");
        assert_eq!(result, "active");
    }

    #[test]
    fn else_runs_when_the_condition_is_false() {
        let result = winners_of("
players 2

def setup(){
    if(1 is 2){
        winner(1)
    } else {
        winner(2)
    }
    if(1 is 1){
        winner(3)
    }
    else {
        winner(4)
    }
}
");
        assert_eq!(result, "active\nwinners: 2, 3");
    }

    #[test]
    fn else_if_chains_pick_the_first_match() {
        let result = winners_of("
players 2

def classify(n){
    if(n is 1){
        return(10)
    } else if(n is 2){
        return(20)
    } else {
        return(30)
    }
}

def setup(){
    winner(classify(1))
    winner(classify(2))
    winner(classify(3))
}
");
        assert_eq!(result, "active\nwinners: 10, 20, 30");
    }

    #[test]
    fn winner_ignores_values_that_are_not_numbers() {
        let result = winners_of("
players 2

def setup(){
    winner(nobody)
    winner(2)
}
");
        assert_eq!(result, "active\nwinners: 2");
    }

    #[test]
    fn a_transfer_count_moves_that_many_cards_to_each_destination() {
        let mut game = game_from_source("
players 3
stack middle

def setup(){
    deck > middle 4
    deck > players 3
}
");
        game.start();

        assert_eq!(game.show("middle"), "king diamonds, queen diamonds, jack diamonds, ten diamonds");
        assert_eq!(game.show("players"), "player 1 (cards: 3), player 2 (cards: 3), player 3 (cards: 3)");
        assert_eq!(game.show("player 1"), "nine diamonds, six diamonds, three diamonds");
    }

    #[test]
    fn a_transfer_count_larger_than_the_source_moves_what_there_is() {
        let mut game = game_from_source("
players 2
stack a
stack b

def setup(){
    deck > a 2
    a > b 5
}
");
        game.start();

        assert_eq!(game.show("a"), "");
        assert_eq!(game.show("b"), "queen diamonds, king diamonds");
    }

    #[test]
    fn taking_moves_exactly_the_given_cards() {
        let mut game = game_from_source("
players 2
stack middle
stack pile

def is_ace(card){
    return(card:rank is Ace)
}

def setup(){
    deck > middle taking filter(deck, is_ace)
    middle > pile taking filter(middle, is_ace)
}
");
        game.start();

        assert_eq!(game.show("middle"), "");
        assert_eq!(game.show("pile"), "ace spades, ace hearts, ace clubs, ace diamonds");
        assert_eq!(game.show("players"), "player 1 (cards: 0), player 2 (cards: 0)");
    }

    #[test]
    fn taking_cards_that_are_not_all_there_moves_nothing_and_stops_the_function() {
        let mut game = game_from_source("
players 2
stack middle
stack pile

def is_ace(card){
    return(card:rank is Ace)
}

def setup(){
    deck > middle 1
    middle > pile taking filter(deck, is_ace)
    end()
}
");
        game.start();

        assert_eq!(game.show("middle"), "king diamonds");
        assert_eq!(game.show("pile"), "");
        assert_eq!(game.show("game"), "active");
    }

    #[test]
    fn a_transfer_with_an_unknown_stack_stops_the_function() {
        let mut game = game_from_source("
players 2

def setup(){
    deck > nowhere
    end()
}
");
        game.start();

        assert_eq!(game.show("game"), "active");
    }

    #[test]
    fn cards_in_stack_checks_every_card_is_present() {
        let result = winners_of("
players 2
stack middle

def is_king(card){
    return(card:rank is King)
}

def setup(){
    deck > middle 2
    if(cards_in_stack(filter(middle, is_king), middle)){
        winner(1)
    }
    if(cards_in_stack(filter(deck, is_king), middle)){
        winner(2)
    }
    if(cards_in_stack(middle, deck)){
        winner(3)
    }
}
");
        assert_eq!(result, "active\nwinners: 1");
    }

    #[test]
    fn per_player_stacks_can_be_used_through_a_player_object() {
        let mut game = game_from_source("
players 2
stack collection per player

def setup(){
    deck > players 2
}

def player_move(player){
    player:hand > player:collection
    if(count(player:collection) is 1 & count(player:hand) is 1){
        end()
    }
}
");
        game.start();
        game.player_move(2).unwrap();

        assert_eq!(game.show("player 2 collection"), "ten diamonds");
        assert_eq!(game.show("player 2 hand"), "queen diamonds");
        assert_eq!(game.show("player 1 collection"), "");
        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn any_player_can_be_addressed_by_number() {
        let mut game = game_from_source("
players 2
stack collection per player

def setup(){
    deck > players:2:collection 3
    players:2:collection > players:1:hand
    if(count(players:2:collection) is 2 & players:1:id is 1){
        end()
    }
}
");
        game.start();

        assert_eq!(game.show("player 1"), "jack diamonds");
        assert_eq!(game.show("player 2 collection"), "king diamonds, queen diamonds");
        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn unknown_player_stacks_are_reported() {
        let mut game = game_from_source("
players 2

def setup(){
    deck > players:3:hand
    end()
}
");
        game.start();

        assert_eq!(game.show("game"), "active");
        assert_eq!(game.show("player 1 collection"), "player 1 has no stack 'collection'");
    }

    #[test]
    fn cards_have_default_values_from_ace_1_to_king_13() {
        let result = winners_of("
players 2

def is_spade(card){
    return(card:suit is Spades)
}

def setup(){
    winner(sum(filter(deck, is_spade)))
}
");
        assert_eq!(result, "active\nwinners: 91");
    }

    #[test]
    fn a_value_table_overrides_card_values() {
        let result = winners_of("
players 2
values {
    Jack 8, Queen 9
    King 10
}

def is_royal(card){
    return(card:rank is Jack or card:rank is Queen or card:rank is King)
}

def is_spade(card){
    return(card:suit is Spades)
}

def setup(){
    winner(sum(filter(filter(deck, is_spade), is_royal)))
    winner(sum(filter(deck, is_spade)))
}
");
        // royals 8 + 9 + 10; all spades 1..10 + 27
        assert_eq!(result, "active\nwinners: 27, 82");
    }

    #[test]
    fn arithmetic_and_ordering_work_on_numbers() {
        let result = winners_of("
players 2

def setup(){
    winner(1 + 2 - 4)
    if(3 > 2 & 2 < 3 & 1 + 1 is 2){
        winner(10)
    }
    if(3 < 2 or 2 > 3){
        winner(20)
    }
    if(count(deck) - 2 is 50){
        winner(30)
    }
}
");
        assert_eq!(result, "active\nwinners: -1, 10, 30");
    }

    #[test]
    fn arithmetic_on_non_numbers_is_not_true() {
        let result = winners_of("
players 2

def setup(){
    if(deck > 3){
        winner(1)
    }
    winner(2)
}
");
        assert_eq!(result, "active\nwinners: 2");
    }
}
