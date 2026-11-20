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
    pub player_stacks: Vec<String>,
    // overrides from a `values { ... }` table, by rank name
    pub card_values: Vec<(String, f64)>,
    pub card_stacks: Vec<String>,
    pub current_player: usize,
}

#[derive(Clone, Debug)]
// The game's functions by name. `setup` and `player_move` are called by
// the interpreter; the rest are called by the game itself.
pub struct Callbacks {
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

// A move as typed by a player, e.g. `2 take 7h / 3c 4s`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MoveInput {
    // defaults to the current player
    pub player: Option<usize>,
    pub name: Option<String>,
    // card groups separated by `/`: move:cards, then move:target
    pub groups: Vec<Vec<Card>>
}

// The parts of the game state a move can change, so a failed move can be undone
struct Snapshot {
    status: GameState,
    deck: Vec<Card>,
    winners: Vec<f64>,
    current_player: usize,
    players: Vec<Player>,
    card_stacks: HashMap<String, Vec<Card>>
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
    card_values: HashMap<String, f64>,
    call_stack: Vec<HashMap<String, ArgumentValue>>,
    // the player and move objects of the move being made, for valid_moves
    current_move: Option<(ArgumentValue, ArgumentValue)>,
    // names of the game functions being run, innermost last
    function_names: Vec<String>,
    // why the most recent check, transfer or move failed
    failure: Option<String>,
    // true when the failure is the game's own message from check(rule, message)
    failure_is_message: bool
}

impl Runtime {
    pub fn new(initial_values: InitialValues, callbacks: Callbacks) -> Runtime {

        let mut card_stacks: HashMap<String, Vec<Card>> = HashMap::new();
        for stack in initial_values.card_stacks.iter() {
            card_stacks.insert(stack.to_string(), vec![]);
        }

        Runtime {
            status: GameState::Pending,
            deck:  standard_deck(),
            winners: vec![],
            current_player: initial_values.current_player,
            call_stack: vec![],
            current_move: None,
            function_names: vec![],
            failure: None,
            failure_is_message: false,
            card_stacks,
            card_values: initial_values.card_values.iter().cloned().collect(),
            players: Self::generate_players(initial_values.players, &initial_values.player_stacks),
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
                let argument = match f.arguments.first() {
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
                let c = match f.arguments.first() {
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
            "sum" => {
                let total = match f.arguments.first().map(|a| self.resolve_expression(a)) {
                    Some(PrimitiveValue::Stack(cards)) => cards.iter().map(|c| self.card_value(c)).sum(),
                    _ => {
                        eprintln!("sum: expected a stack of cards");
                        0.0
                    }
                };
                Some(PrimitiveValue::Number(total))
            },
            "valid_moves" => Some(PrimitiveValue::Bool(self.valid_moves(&f.arguments))),
            "cards_in_stack" => {
                let cards = f.arguments.first().map(|a| self.resolve_expression(a));
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
                let stack = match f.arguments.first().map(|a| self.resolve_expression(a)) {
                    Some(PrimitiveValue::Stack(s)) => s,
                    _ => return Some(PrimitiveValue::Stack(vec![]))
                };
                let definition = match f.arguments.get(1) {
                    Some(Expression::Symbol(name)) => self.callbacks.definitions.get(name).cloned(),
                    _ => None
                };
                match definition {
                    Some(d) => Some(PrimitiveValue::Stack(self.filter(stack, d))),
                    None => {
                        eprintln!("filter: expected a function name as the second argument");
                        Some(PrimitiveValue::Stack(vec![]))
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

    fn call_user_function(&mut self, definition: &Definition, arguments: &[Expression]) -> Option<PrimitiveValue> {
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
        self.function_names.push(definition.name.clone());
        let result = self.handle_statements(&definition.body).value();
        self.function_names.pop();
        self.call_stack.pop();
        Some(result)
    }

    // calls a game function with values that are already resolved
    fn call_with_values(&mut self, definition: &Definition, values: Vec<ArgumentValue>) -> Flow {
        if self.call_stack.len() >= MAX_CALL_DEPTH {
            eprintln!("maximum call depth exceeded in '{}'", definition.name);
            return Flow::Return(PrimitiveValue::Bool(false));
        }
        let call_stack_frame = definition.arguments.iter().cloned().zip(values).collect();
        self.call_stack.push(call_stack_frame);
        self.function_names.push(definition.name.clone());
        let result = self.handle_statements(&definition.body);
        self.function_names.pop();
        self.call_stack.pop();
        result
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

    pub fn player_move(&mut self, input: MoveInput) -> Result<(), String> {
        match self.status {
            GameState::Pending => return Err("the game has not started - use 'start'".to_string()),
            GameState::GameOver => return Err("the game is over".to_string()),
            GameState::Active => ()
        }

        let p_move = match self.callbacks.definitions.get("player_move").cloned() {
            Some(m) => m,
            None => return Err("this game does not define player_move".to_string())
        };

        let n = input.player.unwrap_or(self.current_player);
        if n < 1 || n > self.players.len() {
            return Err(format!("there is no player {} (players: {})", n, self.players.len()));
        }

        if input.groups.len() > 2 {
            return Err("a move can have at most two groups of cards (cards / target)".to_string());
        }

        let player = Self::build_player_object(self.players[n - 1].clone());
        let move_object = Self::build_move_object(&input);

        let mut call_stack_frame = HashMap::new();
        if let Some(arg) = p_move.arguments.first() {
            call_stack_frame.insert(arg.clone(), player.clone());
        }
        if let Some(arg) = p_move.arguments.get(1) {
            call_stack_frame.insert(arg.clone(), move_object.clone());
        }

        // a move either happens completely or not at all
        let before = self.snapshot();
        self.failure = None;
        self.current_move = Some((player, move_object));
        self.call_stack.push(call_stack_frame);
        self.function_names.push(p_move.name.clone());
        let result = self.handle_statements(&p_move.body);
        self.function_names.pop();
        self.call_stack.pop();
        self.current_move = None;

        if Self::failed(&result) {
            self.restore(before);
            if self.failure_is_message {
                if let Some(message) = self.failure.take() {
                    return Err(message);
                }
            }
            return Err(match self.failure.take() {
                Some(reason) => format!("not allowed: {}", reason),
                None => "not allowed".to_string()
            });
        }
        Ok(())
    }

    fn build_move_object(input: &MoveInput) -> ArgumentValue {
        let group = |i: usize| PrimitiveValue::Stack(input.groups.get(i).cloned().unwrap_or_default());
        let mut move_object = HashMap::new();
        move_object.insert("name".to_string(), PrimitiveValue::String(input.name.clone().unwrap_or_default()));
        move_object.insert("cards".to_string(), group(0));
        move_object.insert("target".to_string(), group(1));
        ArgumentValue::Obj(move_object)
    }

    // a function failed if a check or transfer stopped it, or it returned false
    fn failed(flow: &Flow) -> bool {
        matches!(flow, Flow::Return(PrimitiveValue::Bool(false)))
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            status: self.status.clone(),
            deck: self.deck.clone(),
            winners: self.winners.clone(),
            current_player: self.current_player,
            players: self.players.clone(),
            card_stacks: self.card_stacks.clone()
        }
    }

    fn restore(&mut self, snapshot: Snapshot) {
        self.status = snapshot.status;
        self.deck = snapshot.deck;
        self.winners = snapshot.winners;
        self.current_player = snapshot.current_player;
        self.players = snapshot.players;
        self.card_stacks = snapshot.card_stacks;
    }

    // valid_moves(take, drop): runs the move the player named, or if they
    // didn't name one, the first of these that succeeds
    fn valid_moves(&mut self, arguments: &[Expression]) -> bool {
        let (player, move_object) = match self.current_move.clone() {
            Some(m) => m,
            None => {
                eprintln!("valid_moves can only be used while a player is making a move");
                return false;
            }
        };

        let mut moves = vec![];
        for argument in arguments {
            match argument {
                Expression::Symbol(name) => match self.callbacks.definitions.get(name) {
                    Some(d) => moves.push(d.clone()),
                    None => {
                        eprintln!("valid_moves: no function called '{}'", name);
                        return false;
                    }
                },
                other => {
                    eprintln!("valid_moves: expected function names, got {:?}", other);
                    return false;
                }
            }
        }

        let requested = match &move_object {
            ArgumentValue::Obj(o) => match o.get("name") {
                Some(PrimitiveValue::String(s)) if !s.is_empty() => Some(s.clone()),
                _ => None
            },
            _ => None
        };

        let values = vec![player, move_object];
        match requested {
            Some(name) => match moves.iter().find(|d| d.name == name) {
                Some(definition) => !Self::failed(&self.call_with_values(definition, values)),
                None => {
                    let names: Vec<String> = moves.iter().map(|d| d.name.clone()).collect();
                    self.failure = Some(format!("unknown move '{}' - valid moves: {}", name, names.join(", ")));
                    self.failure_is_message = false;
                    false
                }
            },
            None => {
                let mut reasons = vec![];
                for definition in moves.iter() {
                    let before = self.snapshot();
                    self.failure = None;
                    if !Self::failed(&self.call_with_values(definition, values.clone())) {
                        return true;
                    }
                    self.restore(before);
                    reasons.push(self.failure.take().unwrap_or_else(|| format!("{} failed", definition.name)));
                }
                self.failure = Some(format!("no valid move - {}", reasons.join("; ")));
                self.failure_is_message = false;
                false
            }
        }
    }

    pub fn setup(&mut self) {
        self.status = GameState::Active;
        if let Some(setup) = self.callbacks.definitions.get("setup").cloned() {
            self.failure = None;
            self.function_names.push(setup.name.clone());
            let result = self.handle_statements(&setup.body);
            self.function_names.pop();
            if Self::failed(&result) {
                if let Some(reason) = self.failure.take() {
                    eprintln!("setup stopped: {}", reason);
                }
            }
        }
    }

    // records why something failed, e.g. "check failed in take: count(move:cards) is 1"
    fn fail(&mut self, what: &str, detail: String) {
        let function = self.function_names.last().cloned().unwrap_or_else(|| "game".to_string());
        self.failure = Some(format!("{} in {}: {}", what, function, detail));
        self.failure_is_message = false;
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
                    let result = self.handle_function_call(f);
                    // a move that isn't valid stops player_move, like a failed check
                    if f.name == "valid_moves" && result == Some(PrimitiveValue::Bool(false)) {
                        return Flow::Return(PrimitiveValue::Bool(false));
                    }
                },
                Statement::IfStatement(i) => {
                    if let Flow::Return(v) = self.handle_if_statement(i) {
                        return Flow::Return(v);
                    }
                },
                Statement::Every(e) => {
                    if let Flow::Return(v) = self.handle_every_loop(e) {
                        return Flow::Return(v);
                    }
                },
                Statement::CheckStatement(c) => {
                    if !self.resolve_to_bool(&c.expression) {
                        match &c.message {
                            Some(message) => {
                                self.failure = Some(message.clone());
                                self.failure_is_message = true;
                            },
                            None => self.fail("check failed", c.expression.to_string())
                        }
                        return Flow::Return(PrimitiveValue::Bool(false));
                    }
                },
                Statement::ReturnStatement(r) => {
                    let value = self.resolve_expression(&r.expression);
                    if value == PrimitiveValue::Bool(false) {
                        self.fail("returned false", format!("return({})", r.expression));
                    }
                    return Flow::Return(value);
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

                // players:2:collection - any player's field by number
                if let ["players", n, field] = components.as_slice() {
                    let value = n.parse::<usize>().ok()
                        .and_then(|n| n.checked_sub(1))
                        .and_then(|i| self.player_field(i, field));
                    return match value {
                        Some(v) => v,
                        None => {
                            eprintln!("unknown player field '{}'", s);
                            PrimitiveValue::Bool(false)
                        }
                    };
                }

                match self.find_in_call_stack(components[0]) {
                    Some(ArgumentValue::Obj(o)) if components.len() > 1 => {
                        // player fields are read live: stacks change during a move
                        if let Some(index) = Self::player_index_of(&o) {
                            if let Some(v) = self.player_field(index, components[1]) {
                                return v;
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
            Expression::FunctionCall(f) => self.handle_function_call(f).unwrap_or(PrimitiveValue::Bool(false)),
            Expression::Number(n) => PrimitiveValue::Number(*n),
            Expression::Bool(_)
                | Expression::Comparison(_)
                | Expression::And(_)
                | Expression::Or(_)
                | Expression::Every(_) => PrimitiveValue::Bool(self.resolve_to_bool(expression)),
            Expression::Binary(b) => self.evaluate_binary(b)
        }
    }

    fn generate_players(n: u32, stack_names: &[String]) -> Vec<Player>{
        (0..n).map(|i| Player::with_stacks(i + 1, stack_names)).collect()
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

    fn build_card_object(&self, card: Card) -> ArgumentValue {
        let mut card_object = HashMap::new();
        card_object.insert("rank".to_string(), PrimitiveValue::String(card.get_rank_str()));
        card_object.insert("suit".to_string(), PrimitiveValue::String(card.get_suit_str()));
        card_object.insert("value".to_string(), PrimitiveValue::Number(self.card_value(&card)));
        ArgumentValue::Obj(card_object)
    }

    // Ace 1 ... King 13, unless the game's `values` table says otherwise
    fn card_value(&self, card: &Card) -> f64 {
        match self.card_values.get(&card.get_rank_str()) {
            Some(v) => *v,
            None => card.default_value()
        }
    }

    fn evaluate_binary(&mut self, b: &Binary) -> PrimitiveValue {
        let left = self.resolve_expression(&b.left);
        let right = self.resolve_expression(&b.right);
        let (l, r) = match (&left, &right) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => (*l, *r),
            _ => {
                eprintln!("{:?} needs two numbers, got {:?} and {:?}", b.op, left, right);
                return PrimitiveValue::Bool(false);
            }
        };
        match b.op {
            BinaryOp::Add => PrimitiveValue::Number(l + r),
            BinaryOp::Subtract => PrimitiveValue::Number(l - r),
            BinaryOp::LessThan => PrimitiveValue::Bool(l < r),
            BinaryOp::GreaterThan => PrimitiveValue::Bool(l > r),
            BinaryOp::LessOrEqual => PrimitiveValue::Bool(l <= r),
            BinaryOp::GreaterOrEqual => PrimitiveValue::Bool(l >= r)
        }
    }

    // the players, or the cards in a pile, that `every` goes through
    fn every_items(&mut self, kind: &EveryKind) -> Vec<ArgumentValue> {
        match kind {
            EveryKind::Player => self.players.clone().into_iter().map(Self::build_player_object).collect(),
            EveryKind::Card(pile) => match self.resolve_expression(pile) {
                PrimitiveValue::Stack(cards) => cards.iter().map(|c| self.build_card_object(*c)).collect(),
                other => {
                    eprintln!("every card: '{}' is not a pile of cards (got {:?})", pile, other);
                    vec![]
                }
            }
        }
    }

    // every player p { ... }: a check, return or failed transfer inside the
    // loop ends the whole function, as it would outside a loop
    fn handle_every_loop(&mut self, every: &EveryLoop) -> Flow {
        for item in self.every_items(&every.kind) {
            let mut frame = HashMap::new();
            frame.insert(every.variable.clone(), item);
            self.call_stack.push(frame);
            let flow = self.handle_statements(&every.body);
            self.call_stack.pop();
            if let Flow::Return(v) = flow {
                return Flow::Return(v);
            }
        }
        Flow::Continue
    }

    // every player p: condition - true when it holds for all of them
    fn every_test(&mut self, every: &EveryTest) -> bool {
        for item in self.every_items(&every.kind) {
            let mut frame = HashMap::new();
            frame.insert(every.variable.clone(), item);
            self.call_stack.push(frame);
            let holds = self.resolve_to_bool(&every.condition);
            self.call_stack.pop();
            if !holds {
                return false;
            }
        }
        true
    }

    // returns false when the transfer can't happen
    fn handle_transfer(&mut self, t: &Transfer) -> bool {
        let from = self.get_stack(&t.from);
        let to = self.get_stack(&t.to);

        for (name, stack) in [(&t.from, &from), (&t.to, &to)].iter() {
            if stack.is_none() {
                let detail = format!("{} (unknown stack '{}')", t, name);
                self.fail("transfer failed", detail);
                return false;
            }
        }

        if let Some(TransferCount::Taking(cards)) = &t.count {
            return self.transfer_specific_cards(t, from, to, cards);
        }

        let (new_from, new_to) = match transfer(from, to, t.count.as_ref()) {
            Some((a, b)) => (a, b),
            None => {
                self.fail("transfer failed", t.to_string());
                return false;
            }
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
                self.fail("transfer failed", format!("{} ('taking' needs a single stack on each side)", t));
                return false;
            }
        };

        let wanted = match self.resolve_expression(cards) {
            PrimitiveValue::Stack(c) => c,
            other => {
                self.fail("transfer failed", format!("{} ('taking' needs cards, got {:?})", t, other));
                return false;
            }
        };

        for card in wanted.iter() {
            match from_stack.iter().position(|c| c == card) {
                Some(i) => { from_stack.remove(i); },
                // not all the cards are there, so move none of them
                None => {
                    self.fail("transfer failed", format!("{} ({} is not in {})", t, card, t.from));
                    return false;
                }
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
            Expression::Every(e) => self.every_test(e),
            // values and function calls count as true only when they are the boolean true
            _ => self.resolve_expression(expression) == PrimitiveValue::Bool(true)
        }
    }

    fn get_stack(&self, stack_key: &str) -> Option<TransferTarget> {
        match stack_key {
            "deck" => Some(TransferTarget::Stack(self.deck.clone())),
            "players" => Some(TransferTarget::StackList(self.players.iter().map(|p| p.get_hand()).collect())),
            key => match self.player_stack_ref(key) {
                Some((i, name)) => self.players[i].get_stack(&name).map(TransferTarget::Stack),
                None => self.card_stacks.get(key).map(|s| TransferTarget::Stack(s.clone()))
            }
        }
    }

    fn set_stack(&mut self, stack_key: &str, stack: TransferTarget) {
        match stack_key {
            "deck" => self.deck = stack.get_stack(0),
            "players" => self.players.iter_mut().enumerate().for_each(|(n, p)| {
                p.set_hand(stack.get_stack(n))
            }),
            key => match self.player_stack_ref(key) {
                Some((i, name)) => { self.players[i].set_stack(&name, stack.get_stack(0)); },
                None => {
                    if self.card_stacks.contains_key(key) {
                        self.card_stacks.insert(key.to_string(), stack.get_stack(0));
                    }
                }
            }
        }
    }

    // Finds which player (by index) and which of their stacks a name refers to:
    // `p:hand` or `p:collection` where `p` is a player object in scope,
    // or `players:2:hand` for any player by number.
    fn player_stack_ref(&self, key: &str) -> Option<(usize, String)> {
        let parts: Vec<&str> = key.split(':').collect();
        let (index, field) = match parts.as_slice() {
            ["players", n, field] => (n.parse::<usize>().ok()?.checked_sub(1)?, *field),
            [object, field] => (self.player_index_in_scope(object)?, *field),
            _ => return None
        };
        let player = self.players.get(index)?;
        player.get_stack(field)?;
        Some((index, field.to_string()))
    }

    fn player_index_in_scope(&self, name: &str) -> Option<usize> {
        match self.find_in_call_stack(name) {
            Some(ArgumentValue::Obj(o)) => Self::player_index_of(&o),
            _ => None
        }
    }

    fn player_index_of(obj: &HashMap<String, PrimitiveValue>) -> Option<usize> {
        match obj.get(INTERNAL_REF) {
            Some(PrimitiveValue::String(s)) => s.split(':').nth(1)?.parse::<usize>().ok(),
            _ => None
        }
    }

    // a field of a player, read live so it reflects transfers made during a move
    fn player_field(&self, index: usize, field: &str) -> Option<PrimitiveValue> {
        let player = self.players.get(index)?;
        if field == "id" {
            return Some(PrimitiveValue::Number(player.get_id() as f64));
        }
        player.get_stack(field).map(PrimitiveValue::Stack)
    }

    pub fn find_custom_item(&self, key: &str) -> Option<Vec<Card>> {
        self.card_stacks.get(key).map(|v| v.to_vec())
    }

    fn find_in_call_stack(&self, key: &str) -> Option<ArgumentValue> {
        for frame in self.call_stack.iter().rev(){
            let result = frame.get(key);
            if let Some(r) = result { return Some(r.clone()) }
        }
        None
    }

    fn filter(&mut self, stack: Vec<Card>, function: Definition) -> Vec<Card> {
        let card_arg = match function.arguments.first() {
            Some(arg) => arg,
            None => "card"
        }.to_string();

        stack.iter().filter(|&card|{
            let mut call_stack_frame = HashMap::new();
            let card_obj = self.build_card_object(*card);
            call_stack_frame.insert(card_arg.clone(), card_obj);
            self.call_stack.push(call_stack_frame);
            self.function_names.push(function.name.clone());
            let keep_card = self.handle_statements(&function.body).value();
            self.function_names.pop();
            self.call_stack.pop();
            match keep_card {
                PrimitiveValue::Bool(b) => b,
                _ => false
            }
        }).copied().collect()
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
            arguments: vec!["card".to_string()],
            body: vec![return_statement]
        };

        let initial_values = InitialValues{
            players: 1,
            player_stacks: vec![],
            card_values: vec![],
            card_stacks: vec![],
            current_player: 1,
        };

        let callbacks = Callbacks{
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
            arguments: vec!["card".to_string()],
            body: vec![return_statement]
        };

        let initial_values = InitialValues{
            players: 1,
            player_stacks: vec![],
            card_values: vec![],
            card_stacks: vec![],
            current_player: 1,
        };

        let callbacks = Callbacks{
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
            arguments: vec!["card".to_string()],
            body: vec![return_statement]
        };

        let initial_values = InitialValues{
            players: 1,
            player_stacks: vec![],
            card_values: vec![],
            card_stacks: vec![],
            current_player: 1,
        };

        let callbacks = Callbacks{
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
            arguments: vec!["card".to_string()],
            body: vec![return_statement]
        };

        let initial_values = InitialValues{
            players: 1,
            player_stacks: vec![],
            card_values: vec![],
            card_stacks: vec![],
            current_player: 1,
        };

        let callbacks = Callbacks{
            definitions: HashMap::new()
        };

        let mut runtime = Runtime::new(initial_values, callbacks);

        let filtered_cards = runtime.filter(cards, func);

        assert_eq!(filtered_cards.len(), 48);
    }
}