use crate::ast::*;
use crate::cards::{standard_deck, Card, Player};
use std::fmt::Display;
use crate::runtime::{transfer::{transfer, TransferTarget}, std::{shuffle, end, winner}};
use std::collections::HashMap;

#[derive(Clone)]
enum ArgumentValue {
    Number(usize)
}

#[derive(Clone, PartialEq)]
pub enum GameState {
    Pending,
    Active,
    GameOver
}

#[derive(Clone)]
pub struct Game {
    name: Option<String>,
    deck: Vec<Card>,
    players: Vec<Player>,
    setup: Vec<Statement>,
    player_move: Vec<Statement>,
    ast: Vec<Statement>,
    call_stack: Vec<HashMap<String, ArgumentValue>>,
    card_stacks: HashMap<String, Vec<Card>>,
    status: GameState,
    winners: Vec<f64>
}

impl Game {
    pub fn new(ast: Vec<Statement>) -> Game {
        let deck = vec!();
        let name = None;
        let players = vec!();
        let mut setup = vec!();
        let mut player_move = vec!();
        let call_stack = vec!();
        let card_stacks = HashMap::new();
        let status = GameState::Pending;
        let winners = vec!();

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
                        "player_move" => { player_move = b.to_vec(); },
                        _ => ()
                    }
                },
                _ => ()
            }
        }
        let mut game = Game{
            deck,
            name,
            players,
            setup,
            ast,
            player_move,
            call_stack,
            card_stacks,
            status,
            winners
        };
        game.initialise_declarations();
        game
    }

    fn initialise_declarations(&mut self) {
        self.deck = standard_deck();
        self.card_stacks = HashMap::new();
        self.winners = vec!();
        for statement in self.ast.iter() {
            match statement {
                Statement::Declaration(Declaration{
                    key: GlobalKey::Name,
                    value: Expression::Symbol(v)
                }) => {
                    self.name = Some(v.to_string());
                },
                Statement::Declaration(Declaration{
                    key: GlobalKey::Players,
                    value: Expression::Number(n)
                }) => {
                    self.players = Self::generate_players(*n as i32);
                },
                Statement::Declaration(Declaration{
                    key: GlobalKey::Stack,
                    value: Expression::Symbol(s)
                }) => {
                    self.card_stacks.insert(s.to_string(), vec!());
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
            "game" => {
                let winners = if self.winners.len() > 0 {
                    let w = self.winners.iter().map(|n|{n.to_string()}).collect::<Vec<String>>().join(", ");
                    format!("\nwinners: {}", w)
                } else {
                    "".to_string()
                };
                let status = match self.status {
                    GameState::Pending => "pending", 
                    GameState::Active => "active",
                    GameState::GameOver => "game over"
                };
                format!("{}{}", status, winners)
            },
            _ => self.check_exploded_show(key)
        }
    }

    pub fn start(&mut self) {
        self.status = GameState::Active;
        self.initialise_declarations();
        self.handle_statements(&self.setup.clone());
    }

    pub fn player_move(&mut self, player: usize) {
        if self.status != GameState::Active {
            return;
        }

        let mut call_stack_frame = HashMap::new();
        call_stack_frame.insert("player".to_string(), ArgumentValue::Number(player));
        self.call_stack.push(call_stack_frame);
        self.handle_statements(&self.player_move.clone());
        self.call_stack.pop();
    }

    fn check_exploded_show(&self, key: &str) -> String {
        let instructions: Vec<&str> = key.split(" ").collect();
        match instructions[0] {
            "player" => self.handle_show_player(instructions),
            key => self.find_custom_item(key) //format!("Unknown item: {}", key)
        }
    }

    fn handle_show_player(&self, args: Vec<&str>) -> String {
        let player_num = args[1].parse::<usize>().unwrap_or(1) - 1;
        Self::display_list(&self.players[player_num].get_hand())
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
                Statement::IfStatement(i) => self.handle_if_statement(i),
                _ => ()
            }
        })
    }

    // todo - handle failures
    fn handle_transfer(&mut self, t: &Transfer) {
        let from = self.get_stack(&t.from);
        let to = self.get_stack(&t.to);

        let transfer_result = transfer(from, to, t.count.as_ref());

        let (new_from, new_to) = match transfer_result {
            Some((a, b)) => (a, b),
            _ => return
        };

        self.set_stack(&t.from, new_from);
        self.set_stack(&t.to, new_to);
    }

    fn handle_function_call(&mut self, f: &FunctionCall) {
        match f.name.as_str() {
            "end" => end(&mut self.status),
            "shuffle" => shuffle(&mut self.deck),
            "winner" => {
                let player_id = self.resolve_expression(&f.arguments[0]).to_number();
                winner(&mut self.winners, player_id)
            },
            _ => ()
        }        
    }

    fn handle_if_statement(&mut self, i: &IfStatement) {
        match i.expression {
            Expression::Bool(true) => self.handle_statements(&i.body.clone()),
            _ => ()
        }
    }

    fn get_stack(&self, stack_key: &str) -> Option<TransferTarget> {    
        let instructions: Vec<&str> = stack_key.split(&[' ', ':'][..]).collect();
        match instructions[0] {
            "deck" => Some(TransferTarget::Stack(self.deck.clone())),
            "players" => Some(TransferTarget::StackList(self.players.iter().map(|p| p.get_hand()).collect())),
            "player" => {
                let player = self.find_in_call_stack("player");
                match player {
                    Some(ArgumentValue::Number(n)) => Some(TransferTarget::Stack(self.players[n - 1].get_hand())),
                    _ => None
                }
            },
            key => self.find_custom_stack(key)
        }
    }

    fn set_stack(&mut self, stack_key: &str, stack: TransferTarget) {
        let instructions: Vec<&str> = stack_key.split(&[' ', ':'][..]).collect();
        match instructions[0] {
            "deck" => self.deck = stack.get_stack(0),
            "players" => self.players.iter_mut().enumerate().for_each(|(n, p)| {
                let new_hand = stack.get_stack(n);
                p.set_hand(new_hand)
            }),
            "player" => {
                let player = self.find_in_call_stack("player");
                match player {
                    Some(ArgumentValue::Number(n)) => self.players[n - 1].set_hand(stack.get_stack(0)),
                    _ => ()
                }
            },
            key => { self.card_stacks.insert(key.to_string(), stack.get_stack(0)); }
        }
    }

    fn find_custom_stack(&self, key: &str) -> Option<TransferTarget> {
        let stack_result = self.card_stacks.get(key);
        match stack_result {
            Some(s) => Some(TransferTarget::Stack(s.clone())),
            _ => None
        }
    }

    fn find_custom_item(&self, key: &str) -> String {
        match self.card_stacks.get(key) {
            Some(v) => Self::display_list(v),
            _ => "".to_string()
        }
    }
    fn find_in_call_stack(&self, key: &str) -> Option<ArgumentValue> {
        for frame in self.call_stack.iter().rev(){
            let result = frame.get(key);
            match result {
                Some(r)  => return Some(r.clone()),
                _ => ()

            }
        }
        None
    }

    fn resolve_expression(&self, expression: &Expression) -> Expression {
        match expression {
            Expression::Symbol(_s) => {
                //let components = s.split(&[':']).collect();
                match self.find_in_call_stack("player") {
                    Some(ArgumentValue::Number(n)) => Expression::Number(n as f64),
                    _ => expression.to_owned()
                }
            },
            _ => expression.to_owned()
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
        let definition = Definition{ name, body };
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
                    to: "player hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "active\nwinners: 1");
    }

    #[test]
    fn it_can_apply_a_winner_by_id() {
        let body = vec!(
            Statement::FunctionCall(
                FunctionCall{
                    name: "winner".to_string(),
                    arguments: vec!(Expression::Symbol("player:id".to_string()))
                }
            )
        );

        let name = "player_move".to_owned();
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

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
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
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
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);
        let ast = vec!(statement);

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "active");
    }
}