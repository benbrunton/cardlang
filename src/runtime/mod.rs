mod transfer;
pub mod std;

use self::std::*;
use crate::ast::*;
use crate::cards::{standard_deck, Card, Player};
use ::std::{fmt, collections::HashMap};
use transfer::{transfer, TransferTarget};

#[derive(Clone, PartialEq, Debug)]
pub enum GameState {
    Pending,
    Active,
    GameOver
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            GameState::Pending => write!(f, "pending"),
            GameState::Active => write!(f, "active"),
            GameState::GameOver => write!(f, "game over"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum PrimitiveValue {
    Bool(bool),
    Number(f64),
    Stack(Vec<Card>),
    String(String)
}

#[derive(Clone, Debug)]
pub enum ArgumentValue {
    Obj(HashMap<String, PrimitiveValue>)
}

#[derive(Clone, Debug)]
pub struct InitialValues {
    pub players: u32,
    pub card_stacks: Vec<String>,
    pub current_player: usize,
}

#[derive(Clone, Debug)]
pub struct Callbacks {
    pub player_move: Option<Definition>,
    pub setup: Option<Definition>
}

const INTERNAL_REF: &str = "_ref";

#[derive(Clone, Debug)]
pub struct Runtime {
    callbacks: Callbacks,
    status: GameState,
    deck: Vec<Card>,
    winners: Vec<f64>,
    current_player: usize,
    players: Vec<Player>,
    card_stacks: HashMap<String, Vec<Card>>,
    call_stack: Vec<HashMap<String, ArgumentValue>>
}

impl Runtime {
    pub fn new(initial_values: InitialValues, callbacks: Callbacks) -> Runtime {

        let mut card_stacks: HashMap<String, Vec<Card>> = HashMap::new();
        for stack in initial_values.card_stacks.iter() {
            card_stacks.insert(stack.to_string(), vec!());
        }

        Runtime {
            status: GameState::Pending,
            deck:  standard_deck(),
            winners: vec!(),
            current_player: initial_values.current_player,
            call_stack: vec!(),
            card_stacks,
            players: Self::generate_players(initial_values.players),
            callbacks
        }
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

    pub fn get_status(&self) -> String {
        format!("{}", self.status)
    }

    pub fn get_current_player(&self) -> usize {
        self.current_player
    }

    pub fn get_deck(&self) -> Vec<Card> {
        self.deck.clone()
    }

    pub fn get_players(&self) -> Vec<Player> {
        self.players.clone()
    }

    pub fn get_player(&self, n: usize) -> Player {
        self.players[n].clone()
    }

    pub fn get_winners(&self) -> Vec<f64> {
        self.winners.clone()
    }

    pub fn player_move(&mut self, n: usize) {
        if self.status != GameState::Active {
            return;
        }

        let p_move = self.callbacks.player_move.clone().unwrap();

        let mut call_stack_frame = HashMap::new();
        match p_move.arguments.get(0) {
            Some(arg) => {
                let player = self.players[n - 1].clone();
                call_stack_frame.insert(arg.clone(), Self::build_player_object(player));
            },
            None => ()
        }
        self.call_stack.push(call_stack_frame);
        self.handle_statements(&p_move.body.clone());
        self.call_stack.pop();
    }

    pub fn setup(&mut self) {
        self.status = GameState::Active;
        let setup = self.callbacks.setup.clone();
        match setup {
            Some(setup) => { self.handle_statements(&setup.body.clone()); },
            _ => ()
        }
    }

    fn handle_statements(&mut self, statements: &Vec<Statement>) -> PrimitiveValue {
        let default_return = PrimitiveValue::Bool(false);
        for statement in statements.iter() {
            match statement {
                Statement::Transfer(t) => self.handle_transfer(t),
                Statement::FunctionCall(f) => {
                    let _ = self.handle_function_call(f);
                },
                Statement::IfStatement(i) => self.handle_if_statement(i),
                Statement::CheckStatement(c) => {
                    if !self.resolve_to_bool(&c.expression) {
                        return default_return;
                    }
                },
                Statement::ReturnStatement(r) => {
                    return self.resolve_expression(&r.expression);
                }
                _ => ()
            }
        }

        default_return
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
                    Some(ArgumentValue::Obj(o)) if components.len() > 1 => {
                        match o.get(components[1]){
                            Some(v) => v.clone(),
                            None => PrimitiveValue::Bool(false)
                        }
                    },
                    _ => PrimitiveValue::String(s.to_string())
                }
            },
            Expression::FunctionCall(f) => self.handle_function_call(&f).unwrap_or(PrimitiveValue::Bool(false)),
            Expression::Number(n) => PrimitiveValue::Number(*n),
            Expression::Bool(_) | Expression::Comparison(_) => PrimitiveValue::Bool(self.resolve_to_bool(expression)),
            _ => PrimitiveValue::Bool(false)
        }
    }

    fn generate_players(n: u32) -> Vec<Player>{
        let mut players = vec!();
        for i in 0..n {
            players.push(
                Player::new(i + 1)
            );
        }
        players
    }

    fn build_player_object(player: Player) -> ArgumentValue {
        let id = player.get_id();
        let mut player_object = HashMap::new();
        let internal_ref = format!("players:{}", id as usize - 1);
        player_object.insert(INTERNAL_REF.to_string(), PrimitiveValue::String(internal_ref));
        player_object.insert("id".to_string(), PrimitiveValue::Number(id as f64));
        player_object.insert("hand".to_string(), PrimitiveValue::Stack(player.get_hand()));
        ArgumentValue::Obj(player_object)
    }

    fn build_card_object(card: Card) -> ArgumentValue {
        let mut card_object = HashMap::new();
        card_object.insert("rank".to_string(), PrimitiveValue::String(card.get_rank_str()));
        card_object.insert("suit".to_string(), PrimitiveValue::String(card.get_suit_str()));
        ArgumentValue::Obj(card_object)
    }

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
            key => self.find_dynamic_stack(key)
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
            key => self.set_dynamic_stack(key, stack)
        }
    }

    fn find_dynamic_stack(&self, key: &str) -> Option<TransferTarget> {
        let custom_stack = self.find_custom_stack(key);

        if custom_stack.is_some() {
            return custom_stack;
        }

        let call_stack = self.find_transfer_target_in_call_stack(key);

        if call_stack.is_some() {
            return call_stack;
        }

        return None;
    }

    fn set_dynamic_stack(&mut self, key: &str, stack: TransferTarget) {
        let custom_stack = self.find_custom_stack(key);

        if custom_stack.is_some() {
            self.card_stacks.insert(key.to_string(), stack.get_stack(0));
            return;
        }

        self.set_transfer_target_in_call_stack(key, stack);
    }

    fn find_custom_stack(&self, key: &str) -> Option<TransferTarget> {
        let stack_result = self.card_stacks.get(key);
        match stack_result {
            Some(s) => Some(TransferTarget::Stack(s.clone())),
            _ => None
        }
    }

    fn find_transfer_target_in_call_stack(&self, key: &str) -> Option<TransferTarget> {
        let obj = self.find_in_call_stack(key);
        match obj {
            Some(ArgumentValue::Obj(p)) => {
                match p.get(INTERNAL_REF) {
                    Some(PrimitiveValue::String(s)) => {
                        let parts: Vec<&str> = s.split(":").collect();
                        let i = parts[1].parse::<usize>().unwrap();

                        let stack = self.players[i].get_hand();
                        Some(TransferTarget::Stack(stack.to_vec()))
                    },
                    _ => None
                }
            },
            _ => None
        }
    }

    fn set_transfer_target_in_call_stack(&mut self, key: &str, stack: TransferTarget) {
        let obj = self.find_in_call_stack(key);
        match obj {
            Some(ArgumentValue::Obj(p)) => {
                match p.get(INTERNAL_REF) {
                    Some(PrimitiveValue::String(s)) => {
                        let parts: Vec<&str> = s.split(":").collect();
                        let i = parts[1].parse::<usize>().unwrap();

                        self.players[i].set_hand(stack.get_stack(0));
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }

    pub fn find_custom_item(&self, key: &str) -> Option<Vec<Card>> {
        match self.card_stacks.get(key) {
            Some(v) => Some(v.to_vec()),
            None    => None
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

    pub fn filter(&mut self, stack: Vec<Card>, function: Definition) -> Vec<Card> {
        let card_arg = match function.arguments.get(0) {
            Some(arg) => arg,
            None => "card"
        }.to_string();

        return stack.iter().filter(|&card|{
            let mut call_stack_frame = HashMap::new();
            let card_obj = Self::build_card_object(*card);
            call_stack_frame.insert(card_arg.clone(), card_obj);
            self.call_stack.push(call_stack_frame);
            let keep_card = self.handle_statements(&function.body.clone());
            self.call_stack.pop();
            match keep_card {
                PrimitiveValue::Bool(b) => b,
                _ => false
            }
        }).map(|&card| card.clone()).collect()
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::cards::standard_deck;

    #[test]
    fn primitive_strings_can_be_compared() {
        assert_eq!(PrimitiveValue::String("Ace".to_string()), PrimitiveValue::String("Ace".to_string()))
    }

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

        let initial_values = InitialValues{
            players: 1,
            card_stacks: vec!(),
            current_player: 1,
        };

        let callbacks = Callbacks{
            player_move: None,
            setup: None
        };

        let mut runtime = Runtime::new(initial_values, callbacks);

        let filtered_cards = runtime.filter(cards, func);

        assert_eq!(filtered_cards.len(), 52);
    }

    #[test]
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

        let initial_values = InitialValues{
            players: 1,
            card_stacks: vec!(),
            current_player: 1,
        };

        let callbacks = Callbacks{
            player_move: None,
            setup: None
        };

        let mut runtime = Runtime::new(initial_values, callbacks);

        let filtered_cards = runtime.filter(cards, func);

        assert_eq!(filtered_cards.len(), 0);
    }

    #[test]
    fn filter_executes_a_function_against_a_stack_and_passes_card_to_function() {
        let cards = standard_deck();
        let expression = Expression::Comparison(Box::new(Comparison{
            left: Expression::Symbol("card:rank".to_string()),
            right: Expression::Symbol("Ace".to_string()),
            negative: false
        }));

        let return_statement = Statement::ReturnStatement(ReturnStatement{ expression });
        let func = Definition{
            name: "_".to_string(),
            arguments: vec!("card".to_string()),
            body: vec!(return_statement)
        };

        let initial_values = InitialValues{
            players: 1,
            card_stacks: vec!(),
            current_player: 1,
        };

        let callbacks = Callbacks{
            player_move: None,
            setup: None
        };

        let mut runtime = Runtime::new(initial_values, callbacks);

        let filtered_cards = runtime.filter(cards, func);

        assert_eq!(filtered_cards.len(), 4);
    }
}