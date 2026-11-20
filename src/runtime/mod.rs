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
    Obj(HashMap<String, PrimitiveValue>),
    Value(PrimitiveValue)
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
    pub setup: Option<Definition>,
    pub definitions: HashMap<String, Definition>
}

// The result of running a block of statements: either it ran to the end,
// or a `return` or failing `check` ended the enclosing function.
enum Flow {
    Continue,
    Return(PrimitiveValue)
}

impl Flow {
    fn value(self) -> PrimitiveValue {
        match self {
            Flow::Return(v) => v,
            Flow::Continue => PrimitiveValue::Bool(false)
        }
    }
}

const INTERNAL_REF: &str = "_ref";
const MAX_CALL_DEPTH: usize = 256;

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
                let argument = match f.arguments.get(0) {
                    Some(a) => a,
                    None => {
                        eprintln!("winner: expected a player id");
                        return None;
                    }
                };
                let player_id = match self.resolve_expression(argument) {
                    PrimitiveValue::Number(n) => n,
                    other => {
                        eprintln!("winner: expected a player id, got {:?}", other);
                        return None;
                    }
                };

                winner(&mut self.winners, player_id);
                None
            },
            "count" => {
                let c = match f.arguments.get(0) {
                    Some(a) => {
                        let stack_to_count = self.resolve_expression(a);
                        count(stack_to_count)
                    },
                    None => 0
                };
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
            "cards_in_stack" => {
                let cards = f.arguments.get(0).map(|a| self.resolve_expression(a));
                let stack = f.arguments.get(1).map(|a| self.resolve_expression(a));
                let result = match (cards, stack) {
                    (Some(PrimitiveValue::Stack(cards)), Some(PrimitiveValue::Stack(stack))) => {
                        cards_in_stack(&cards, &stack)
                    },
                    _ => {
                        eprintln!("cards_in_stack: expected two stacks of cards");
                        false
                    }
                };
                Some(PrimitiveValue::Bool(result))
            },
            "filter" => {
                let stack = match f.arguments.get(0).map(|a| self.resolve_expression(a)) {
                    Some(PrimitiveValue::Stack(s)) => s,
                    _ => return Some(PrimitiveValue::Stack(vec!()))
                };
                let definition = match f.arguments.get(1) {
                    Some(Expression::Symbol(name)) => self.callbacks.definitions.get(name).cloned(),
                    _ => None
                };
                match definition {
                    Some(d) => Some(PrimitiveValue::Stack(self.filter(stack, d))),
                    None => {
                        eprintln!("filter: expected a function name as the second argument");
                        Some(PrimitiveValue::Stack(vec!()))
                    }
                }
            },
            name => {
                let definition = match self.callbacks.definitions.get(name) {
                    Some(d) => d.clone(),
                    None => {
                        eprintln!("unknown function '{}'", name);
                        return None;
                    }
                };
                self.call_user_function(&definition, &f.arguments)
            }
        }
    }

    fn call_user_function(&mut self, definition: &Definition, arguments: &Vec<Expression>) -> Option<PrimitiveValue> {
        if self.call_stack.len() >= MAX_CALL_DEPTH {
            eprintln!("maximum call depth exceeded in '{}'", definition.name);
            return None;
        }

        // arguments are resolved in the caller's scope before the new frame is pushed
        let mut call_stack_frame = HashMap::new();
        for (i, arg_name) in definition.arguments.iter().enumerate() {
            let value = match arguments.get(i) {
                Some(expression) => self.resolve_argument(expression),
                None => ArgumentValue::Value(PrimitiveValue::Bool(false))
            };
            call_stack_frame.insert(arg_name.clone(), value);
        }

        self.call_stack.push(call_stack_frame);
        let result = self.handle_statements(&definition.body).value();
        self.call_stack.pop();
        Some(result)
    }

    fn resolve_argument(&mut self, expression: &Expression) -> ArgumentValue {
        // pass whole objects (e.g. a player) through by name
        if let Expression::Symbol(s) = expression {
            if !s.contains(':') {
                if let Some(value) = self.find_in_call_stack(s) {
                    return value;
                }
            }
        }
        ArgumentValue::Value(self.resolve_expression(expression))
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

    pub fn get_player(&self, n: usize) -> Option<Player> {
        self.players.get(n).cloned()
    }

    pub fn get_winners(&self) -> Vec<f64> {
        self.winners.clone()
    }

    pub fn player_move(&mut self, n: usize) -> Result<(), String> {
        match self.status {
            GameState::Pending => return Err("the game has not started - use 'start'".to_string()),
            GameState::GameOver => return Err("the game is over".to_string()),
            GameState::Active => ()
        }

        let p_move = match self.callbacks.player_move.clone() {
            Some(m) => m,
            None => return Err("this game does not define player_move".to_string())
        };

        if n < 1 || n > self.players.len() {
            return Err(format!("there is no player {} (players: {})", n, self.players.len()));
        }

        let mut call_stack_frame = HashMap::new();
        if let Some(arg) = p_move.arguments.get(0) {
            let player = self.players[n - 1].clone();
            call_stack_frame.insert(arg.clone(), Self::build_player_object(player));
        }
        self.call_stack.push(call_stack_frame);
        self.handle_statements(&p_move.body);
        self.call_stack.pop();
        Ok(())
    }

    pub fn setup(&mut self) {
        self.status = GameState::Active;
        let setup = self.callbacks.setup.clone();
        match setup {
            Some(setup) => { self.handle_statements(&setup.body.clone()); },
            _ => ()
        }
    }

    fn handle_statements(&mut self, statements: &[Statement]) -> Flow {
        for statement in statements.iter() {
            match statement {
                Statement::Transfer(t) => {
                    // a transfer that can't happen stops the function, like a failed check
                    if !self.handle_transfer(t) {
                        return Flow::Return(PrimitiveValue::Bool(false));
                    }
                },
                Statement::FunctionCall(f) => {
                    let _ = self.handle_function_call(f);
                },
                Statement::IfStatement(i) => {
                    if let Flow::Return(v) = self.handle_if_statement(i) {
                        return Flow::Return(v);
                    }
                },
                Statement::CheckStatement(c) => {
                    if !self.resolve_to_bool(&c.expression) {
                        return Flow::Return(PrimitiveValue::Bool(false));
                    }
                },
                Statement::ReturnStatement(r) => {
                    return Flow::Return(self.resolve_expression(&r.expression));
                }
                _ => ()
            }
        }

        Flow::Continue
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
                        // a player's hand changes during a move, so read it live
                        if components[1] == "hand" {
                            if let Some(hand) = self.live_player_hand(&o) {
                                return PrimitiveValue::Stack(hand);
                            }
                        }
                        match o.get(components[1]){
                            Some(v) => v.clone(),
                            None => PrimitiveValue::Bool(false)
                        }
                    },
                    Some(ArgumentValue::Value(v)) if components.len() == 1 => v,
                    _ if s == "deck" => PrimitiveValue::Stack(self.deck.clone()),
                    _ => match self.card_stacks.get(s) {
                        Some(stack) => PrimitiveValue::Stack(stack.clone()),
                        None => PrimitiveValue::String(s.to_string())
                    }
                }
            },
            Expression::FunctionCall(f) => self.handle_function_call(&f).unwrap_or(PrimitiveValue::Bool(false)),
            Expression::Number(n) => PrimitiveValue::Number(*n),
            Expression::Bool(_)
                | Expression::Comparison(_)
                | Expression::And(_)
                | Expression::Or(_) => PrimitiveValue::Bool(self.resolve_to_bool(expression))
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

    // returns false when the transfer can't happen
    fn handle_transfer(&mut self, t: &Transfer) -> bool {
        let from = self.get_stack(&t.from);
        let to = self.get_stack(&t.to);

        let mut known = true;
        for (name, stack) in [(&t.from, &from), (&t.to, &to)].iter() {
            if stack.is_none() {
                eprintln!("transfer failed: unknown stack '{}'", name);
                known = false;
            }
        }
        if !known {
            return false;
        }

        if let Some(TransferCount::Taking(cards)) = &t.count {
            return self.transfer_specific_cards(t, from, to, cards);
        }

        let (new_from, new_to) = match transfer(from, to, t.count.as_ref()) {
            Some((a, b)) => (a, b),
            None => return false
        };

        self.set_stack(&t.from, new_from);
        self.set_stack(&t.to, new_to);
        true
    }

    // `from > to taking cards`: moves exactly these cards, or nothing
    fn transfer_specific_cards(
        &mut self,
        t: &Transfer,
        from: Option<TransferTarget>,
        to: Option<TransferTarget>,
        cards: &Expression
    ) -> bool {
        let (mut from_stack, mut to_stack) = match (from, to) {
            (Some(TransferTarget::Stack(f)), Some(TransferTarget::Stack(t))) => (f, t),
            _ => {
                eprintln!("transfer failed: 'taking' needs a single stack on each side");
                return false;
            }
        };

        let wanted = match self.resolve_expression(cards) {
            PrimitiveValue::Stack(c) => c,
            other => {
                eprintln!("transfer failed: 'taking' needs cards, got {:?}", other);
                return false;
            }
        };

        for card in wanted.iter() {
            match from_stack.iter().position(|c| c == card) {
                Some(i) => { from_stack.remove(i); },
                // not all the cards are there, so move none of them
                None => return false
            }
            to_stack.push(*card);
        }

        self.set_stack(&t.from, TransferTarget::Stack(from_stack));
        self.set_stack(&t.to, TransferTarget::Stack(to_stack));
        true
    }

    fn handle_if_statement(&mut self, i: &IfStatement) -> Flow {
        if self.resolve_to_bool(&i.expression) {
            self.handle_statements(&i.body)
        } else {
            self.handle_statements(&i.else_body)
        }
    }

    fn resolve_to_bool(&mut self, expression: &Expression) -> bool {
        match expression {
            Expression::Bool(b) => *b,
            Expression::Comparison(c) => {
                let equal = self.resolve_expression(&c.left) == self.resolve_expression(&c.right);
                equal != c.negative
            },
            Expression::And(c) => self.resolve_to_bool(&c.left) && self.resolve_to_bool(&c.right),
            Expression::Or(c) => self.resolve_to_bool(&c.left) || self.resolve_to_bool(&c.right),
            // values and function calls count as true only when they are the boolean true
            _ => self.resolve_expression(expression) == PrimitiveValue::Bool(true)
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

    fn live_player_hand(&self, obj: &HashMap<String, PrimitiveValue>) -> Option<Vec<Card>> {
        match obj.get(INTERNAL_REF) {
            Some(PrimitiveValue::String(s)) => {
                let parts: Vec<&str> = s.split(":").collect();
                let i = parts.get(1)?.parse::<usize>().ok()?;
                self.players.get(i).map(|p| p.get_hand())
            },
            _ => None
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

    fn filter(&mut self, stack: Vec<Card>, function: Definition) -> Vec<Card> {
        let card_arg = match function.arguments.get(0) {
            Some(arg) => arg,
            None => "card"
        }.to_string();

        return stack.iter().filter(|&card|{
            let mut call_stack_frame = HashMap::new();
            let card_obj = Self::build_card_object(*card);
            call_stack_frame.insert(card_arg.clone(), card_obj);
            self.call_stack.push(call_stack_frame);
            let keep_card = self.handle_statements(&function.body).value();
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
            setup: None,
            definitions: HashMap::new()
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
            setup: None,
            definitions: HashMap::new()
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
            setup: None,
            definitions: HashMap::new()
        };

        let mut runtime = Runtime::new(initial_values, callbacks);

        let filtered_cards = runtime.filter(cards, func);

        assert_eq!(filtered_cards.len(), 4);
    }

    #[test]
    fn negative_comparisons_invert_the_result() {
        let cards = standard_deck();
        let expression = Expression::Comparison(Box::new(Comparison{
            left: Expression::Symbol("card:rank".to_string()),
            right: Expression::Symbol("Ace".to_string()),
            negative: true
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
            setup: None,
            definitions: HashMap::new()
        };

        let mut runtime = Runtime::new(initial_values, callbacks);

        let filtered_cards = runtime.filter(cards, func);

        assert_eq!(filtered_cards.len(), 48);
    }
}