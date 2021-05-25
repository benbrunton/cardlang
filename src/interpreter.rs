use crate::ast::*;
use crate::cards::{standard_deck, Card, Player};
use std::fmt::Display;
use crate::runtime::{transfer::{transfer, TransferTarget}, std::{shuffle, end, winner, count}};
use std::collections::HashMap;

#[derive(Clone)]
enum ArgumentValue {
    Number(usize),
    Obj(HashMap<String, PrimitiveValue>)
}

#[derive(Clone, PartialEq)]
pub enum PrimitiveValue {
    Bool(bool),
    Number(f64),
    Stack(Vec<Card>)
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
    player_move: Definition,
    ast: Vec<Statement>,
    call_stack: Vec<HashMap<String, ArgumentValue>>,
    card_stacks: HashMap<String, Vec<Card>>,
    status: GameState,
    winners: Vec<f64>,
    current_player: usize
}

impl Game {
    pub fn new(ast: Vec<Statement>) -> Game {
        let deck = vec!();
        let name = None;
        let players = vec!();
        let mut setup = vec!();
        let mut player_move = Definition{name: "player_move".to_string(), body: vec!(), arguments: vec!()};
        let call_stack = vec!();
        let card_stacks = HashMap::new();
        let status = GameState::Pending;
        let winners = vec!();
        let current_player = 1;

        for statement in ast.iter() {
            match statement {
                Statement::Definition(
                    d
                ) => {
                    match d.name.as_str() {
                        "setup" => {setup = d.body.to_vec();},
                        "player_move" => player_move = d.clone(),
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
            winners,
            current_player
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
                    key: GlobalKey::CurrentPlayer,
                    value: Expression::Number(n)
                }) => {
                    self.current_player = *n as usize;
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
            "current_player" => {
                format!("{}", self.current_player)
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
        match self.player_move.arguments.get(0) {
            Some(arg) => {
                call_stack_frame.insert("player".to_string(), self.build_player_object(player));
            },
            None => ()
        }
        self.call_stack.push(call_stack_frame);
        self.handle_statements(&self.player_move.body.clone());
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
        for statement in statements.iter() {
            match statement {
                Statement::Transfer(t) => self.handle_transfer(t),
                Statement::FunctionCall(f) => {
                    let _ = self.handle_function_call(f);
                },
                Statement::IfStatement(i) => self.handle_if_statement(i),
                Statement::CheckStatement(c) => {
                    if !self.resolve_to_bool(&c.expression) {
                        return;
                    }
                },
                _ => ()
            }
        }
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

    fn handle_function_call(&mut self, f: &FunctionCall) -> Option<PrimitiveValue>{
        match f.name.as_str() {
            "end" => {
                end(&mut self.status);
                None
            },
            "shuffle" => {
                shuffle(&mut self.deck);
                None
            },
            "winner" => {
                let player_id = match self.resolve_expression(&f.arguments[0]) {
                    PrimitiveValue::Number(n) => n,
                    _ => 0.0
                };

                winner(&mut self.winners, player_id);
                None
            },
            "count" => {
                let stack_to_count = self.resolve_expression(&f.arguments[0]);
                let c = count(stack_to_count);
                Some(PrimitiveValue::Number(c as f64))
            },
            "next_player" => {
                self.current_player = if self.current_player < self.players.len() {
                    self.current_player + 1
                } else {
                    1
                };
                None
            },
            _ => None
        }        
    }

    fn handle_if_statement(&mut self, i: &IfStatement) {
        if self.resolve_to_bool(&i.expression) {
            self.handle_statements(&i.body.clone());
        }
    }

    fn resolve_to_bool(&mut self, expression: &Expression) -> bool {
        match expression {
            Expression::Bool(b) => *b,
            Expression::Comparison(c) => self.resolve_expression(&c.left) == self.resolve_expression(&c.right),
            Expression::And(c) => self.resolve_to_bool(&c.left) && self.resolve_to_bool(&c.right),
            _ => false
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
                    Some(ArgumentValue::Obj(p)) => {
                        match p.get("hand") {
                            Some(PrimitiveValue::Stack(s)) => Some(TransferTarget::Stack(s.to_vec())),
                            _ => None
                        }
                    },
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
                    Some(ArgumentValue::Obj(p)) => {
                        match p.get("id") {
                            Some(PrimitiveValue::Number(n)) => self.players[(*n as usize) - 1].set_hand(stack.get_stack(0)),
                            _ => ()
                        }
                    },
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
            _ => format!("{} not found", key)
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

    fn resolve_expression(&mut self, expression: &Expression) -> PrimitiveValue {
        match expression {
            // todo - could push globals into top of call stack
            Expression::Symbol(s) => {
                if s == "current_player" {
                    return PrimitiveValue::Number(self.current_player as f64);
                }
                let components: Vec<&str> = s.split(&[':'][..]).collect();
                match self.find_in_call_stack(components[0]) {
                    Some(ArgumentValue::Number(n)) => PrimitiveValue::Number(n as f64),
                    Some(ArgumentValue::Obj(o)) if components.len() > 1 => {
                        match o.get(components[1]){
                            Some(v) => v.clone(),
                            None => PrimitiveValue::Bool(false)
                        }
                    },
                    _ => PrimitiveValue::Bool(false)
                }
            },
            Expression::FunctionCall(f) => {
                self.handle_function_call(&f).unwrap_or(PrimitiveValue::Bool(false))
            },
            Expression::Number(n) => PrimitiveValue::Number(*n),
            _ => PrimitiveValue::Bool(false)
        }
    }

    fn build_player_object(&self, n: usize) -> ArgumentValue {
        let player = self.players[n - 1].clone();
        let mut player_object = HashMap::new();
        player_object.insert("id".to_string(), PrimitiveValue::Number(n as f64));
        player_object.insert("hand".to_string(), PrimitiveValue::Stack(player.get_hand()));
        ArgumentValue::Obj(player_object)
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
}