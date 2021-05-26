use crate::ast::*;
use std::fmt::Display;
use crate::runtime::{
    Runtime,
    InitialValues,
    Callbacks
};

#[derive(Clone)]
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
            card_stacks: vec!(),
            current_player: 1 
        };

        let mut callbacks = Callbacks {
            player_move: None,
            setup: None
        };

        for statement in ast.iter() {
            match statement {
                Statement::Definition(
                    d
                ) => {
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
                    let w = winner_list.iter().map(|n|{n.to_string()}).collect::<Vec<String>>().join(", ");
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
        //self.handle_statements(&self.setup.clone());
        self.runtime.setup();
    }

    pub fn player_move(&mut self, player: usize) {
        self.runtime.player_move(player);
    }

    fn check_exploded_show(&self, key: &str) -> String {
        let instructions: Vec<&str> = key.split(" ").collect();
        match instructions[0] {
            "player" => self.handle_show_player(instructions),
            key => self.find_custom_item(key)
        }
    }

    fn handle_show_player(&self, args: Vec<&str>) -> String {
        let player_num = args[1].parse::<usize>().unwrap_or(1) - 1;
        Self::display_list(&self.runtime.get_player(player_num).get_hand())
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
        game.player_move(1);

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
        game.player_move(1);

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
        game.player_move(2);

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
        game.player_move(1);

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
        game.player_move(1);

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
        game.player_move(1);

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

        let if_statement = IfStatement{
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

        let if_statement = IfStatement{
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
            right: Expression::Number(1.0)
        };

        let if_statement = IfStatement{
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
            right: Expression::Number(0.0)
        };

        let if_statement = IfStatement{
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
        game.player_move(1);

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
            right: Expression::Number(26.0)
        };

        let if_statement = IfStatement{
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
        game.player_move(1);

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

        let if_statement = IfStatement{
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
        game.player_move(1);

        let player_hand = game.show("player 1 hand");

        assert_eq!(player_hand, "king diamonds".to_string());
    }
}