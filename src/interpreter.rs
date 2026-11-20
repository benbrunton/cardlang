use crate::ast::*;
use std::fmt::Display;
use std::collections::HashMap;
use crate::cards::{card_code, Card};

// What the CLI can offer when the player presses Tab
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompletionContext {
    // the moves named in the game's valid_moves(...) calls
    pub move_names: Vec<String>,
    // shared piles declared with `stack`
    pub stacks: Vec<String>,
    // piles declared with `stack <name> per player`
    pub player_stacks: Vec<String>,
    pub players: usize,
    // card codes in the current player's hand
    pub hand: Vec<String>,
    // card codes in the shared piles
    pub table: Vec<String>
}

// finds the function names passed to valid_moves(...) anywhere in a body
fn collect_valid_moves(statements: &[Statement], names: &mut Vec<String>) {
    for statement in statements {
        match statement {
            Statement::FunctionCall(f) if f.name == "valid_moves" => {
                for argument in f.arguments.iter() {
                    if let Expression::Symbol(name) = argument {
                        names.push(name.clone());
                    }
                }
            },
            Statement::IfStatement(i) => {
                collect_valid_moves(&i.body, names);
                collect_valid_moves(&i.else_body, names);
            },
            Statement::Every(e) => collect_valid_moves(&e.body, names),
            _ => ()
        }
    }
}
use crate::runtime::{
    MoveInput,
    Runtime,
    InitialValues,
    Callbacks
};

#[derive(Clone, Debug)]
pub struct Game {
    name: Option<String>,
    runtime: Runtime,
    initial_values: InitialValues,
    callbacks: Callbacks
}

impl Game {
    pub fn new(ast: Vec<Statement>) -> Game {
        let mut name = None;

        let mut initial_values = InitialValues{ 
            players: 1,
            player_stacks: vec![],
            card_values: vec![],
            named_values: vec![],
            deck_without: vec![],
            card_stacks: vec![],
            current_player: 1 
        };

        let mut callbacks = Callbacks {
            definitions: HashMap::new()
        };

        for statement in ast.iter() {
            match statement {
                Statement::Definition(d) => {
                    callbacks.definitions.insert(d.name.clone(), d.clone());
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
                Statement::ValueTable(table) => match &table.name {
                    None => initial_values.card_values.extend(table.entries.iter().cloned()),
                    Some(name) => initial_values.named_values.push((name.clone(), table.entries.clone()))
                },
                Statement::DeckWithout(ranks) => {
                    initial_values.deck_without.extend(ranks.iter().cloned());
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
                let winners = if !winner_list.is_empty() {
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

    // What tab completion at the CLI can offer for this game right now
    pub fn completion_context(&self) -> CompletionContext {
        let mut move_names = vec![];
        for definition in self.callbacks.definitions.values() {
            collect_valid_moves(&definition.body, &mut move_names);
        }
        move_names.sort();
        move_names.dedup();

        let started = self.is_started();
        let current = self.runtime.get_current_player();
        let hand = match self.runtime.get_player(current.wrapping_sub(1)) {
            Some(player) if started => player.get_hand().iter().map(card_code).collect(),
            _ => vec![]
        };
        let table = if started {
            self.initial_values.card_stacks.iter()
                .filter_map(|name| self.runtime.find_custom_item(name))
                .flatten()
                .map(|card| card_code(&card))
                .collect()
        } else {
            vec![]
        };

        CompletionContext {
            move_names,
            stacks: self.initial_values.card_stacks.clone(),
            player_stacks: self.initial_values.player_stacks.clone(),
            players: self.runtime.get_players().len(),
            hand,
            table
        }
    }

    pub fn is_started(&self) -> bool {
        self.runtime.get_status() != "pending"
    }

    // A summary of the table, shown after `start` and after each move:
    // whose turn it is, the deck, the shared piles, the current player's
    // hand and how many cards are in each player's own piles.
    pub fn table(&self) -> String {
        let mut lines = vec![];

        let winners = self.runtime.get_winners();
        if self.runtime.get_status() == "game over" {
            if winners.is_empty() {
                lines.push("game over".to_string());
            } else {
                lines.push(format!("game over - winners: {}", Self::display_list(&winners)));
            }
        } else {
            lines.push(format!("player {} to play", self.runtime.get_current_player()));
        }

        // the deck is face down, so only its size is shown
        let deck = self.runtime.get_deck();
        if !deck.is_empty() {
            lines.push(format!("  deck: {}", Self::plural(deck.len(), "card")));
        }

        for name in self.initial_values.card_stacks.iter() {
            if let Some(cards) = self.runtime.find_custom_item(name) {
                lines.push(format!("  {}: {}", name, Self::cards_or_count(&cards)));
            }
        }

        let current = self.runtime.get_current_player();
        if self.runtime.get_status() != "game over" {
            if let Some(player) = self.runtime.get_player(current.wrapping_sub(1)) {
                lines.push(format!("  player {} hand: {}", current, Self::display_list(&player.get_hand())));
            }
        }

        for name in self.initial_values.player_stacks.iter() {
            let counts: Vec<String> = self.runtime.get_players().iter().map(|p| {
                let size = p.get_stack(name).map_or(0, |cards| cards.len());
                format!("player {}: {}", p.get_id(), Self::plural(size, "card"))
            }).collect();
            lines.push(format!("  {}: {}", name, counts.join(", ")));
        }

        lines.join("\n")
    }

    // long piles are summarised so the table stays readable
    fn cards_or_count(cards: &[Card]) -> String {
        match cards.len() {
            0 => "empty".to_string(),
            1..=10 => Self::display_list(cards),
            n => format!("{} cards, top: {}", n, cards[n - 1])
        }
    }

    fn plural(n: usize, word: &str) -> String {
        if n == 1 { format!("1 {}", word) } else { format!("{} {}s", n, word) }
    }

    // a move with no name or cards, as a specific player
    #[cfg(test)]
    pub fn player_move(&mut self, player: usize) -> Result<(), String> {
        self.make_move(MoveInput{ player: Some(player), ..MoveInput::default() })
    }

    pub fn make_move(&mut self, input: MoveInput) -> Result<(), String> {
        self.runtime.player_move(input)
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

    fn display_list<D: Display>(list: &[D]) -> String {
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
        let ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Deck,
                    value: Expression::Symbol("StandardDeck".to_string())
                }
            )
        ];

        let game = Game::new(ast);
        let deck = game.show("deck");
        let split_deck: Vec<&str> = deck.split(",").collect();

        assert_eq!(split_deck[0], "ace spades");
        assert_eq!(split_deck.len(), 52);
    }

    #[test]
    fn it_can_display_a_name() {
        let ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Name,
                    value: Expression::Symbol("turns".to_string())
                }
            )
        ];

        let game = Game::new(ast);
        let name = game.show("name");

        assert_eq!(name, "turns".to_string());
    }

    #[test]
    fn it_can_display_players() {
        let ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(3.0)
                }
            )
        ];

        let game = Game::new(ast);
        let players = game.show("players");

        assert_eq!(players, "player 1 (cards: 0), player 2 (cards: 0), player 3 (cards: 0)".to_string());
    }

    #[test]
    fn it_can_display_a_single_player() {
        let ast = vec![
            Statement::Declaration (
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(1.0)
                }
            )
        ];

        let game = Game::new(ast);
        let players = game.show("players");

        assert_eq!(players, "player 1 (cards: 0)".to_string());
    }

    #[test]
    fn it_can_start_a_game() {
        let mut ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(3.0)
                }
            )
        ];
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec![transfer_statement];
        let definition = Definition{ arguments: vec![], name, body };
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
        let mut ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(3.0)
                }
            )
        ];
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec![transfer_statement];
        let definition = Definition{ arguments: vec![], name, body };
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
        let mut ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(3.0)
                }
            )
        ];
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None; //Some(TransferModifier::Alternate);
        let count = Some(TransferCount::End);
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec![transfer_statement];
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();

        let deck = game.show("deck");
        assert_eq!(&deck, "");
    }

    #[test]
    fn it_can_show_player_hand(){
        let mut ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(1.0)
                }
            )
        ];
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None; //Some(TransferModifier::Alternate);
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec![transfer_statement];
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();

        let hand = game.show("player 1 hand");
        assert_eq!(&hand, "king diamonds");
    }

    #[test]
    fn it_can_show_multiple_player_hand(){
        let mut ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(2.0)
                }
            )
        ];
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None; //Some(TransferModifier::Alternate);
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec![transfer_statement];
        let definition = Definition{ arguments: vec![], name, body };
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
        let body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "shuffle".to_string(),
                    arguments: vec!(Expression::Symbol("deck".to_string()))
                }
            )
        ];

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

        let mut game = Game::new(ast);
        game.start();

        let usual_order = Game::display_list(&standard_deck());
        let deck = game.show("deck");

        assert_ne!(deck, usual_order);
    }

    #[test]
    fn it_can_make_a_move() {
        let body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "shuffle".to_string(),
                    arguments: vec!(Expression::Symbol("deck".to_string()))
                }
            )
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);

        let ast = vec![statement];

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

        let body = vec![
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "player:hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!["player".to_string()], name, body };
        let statement = Statement::Definition(definition);

        let ast = vec![
            players,
            statement
        ];

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

        let body = vec![
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "player:hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!["player".to_string()], name, body };
        let statement = Statement::Definition(definition);

        let ast = vec![
            players,
            statement
        ];

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
        let mut ast = vec![
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
        ];
        let from = "deck".to_owned();
        let to = "middle".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec![transfer_statement];
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let mut game = Game::new(ast);
        game.start();

        let middle = game.show("middle");

        assert_eq!(&middle, "king diamonds");
    }

    #[test]
    fn it_can_show_info_about_the_game() {
        let ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Deck,
                    value: Expression::Symbol("StandardDeck".to_string())
                }
            )
        ];

        let game = Game::new(ast);
        let display = game.show("game");

        assert_eq!(display, "pending"); 
    }

    #[test]
    fn it_can_end_a_game() {
        let body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

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

        let body = vec![
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "player hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);

        let ast = vec![
            players,
            statement
        ];

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

        let body = vec![
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "player hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);

        let body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_owned(),
                    arguments: vec!()
                }
            )
        ];
        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let setup = Statement::Definition(definition);

        let ast = vec![
            players,
            statement,
            setup
        ];

        let mut game = Game::new(ast);
        game.start();
        assert!(game.player_move(1).is_err());

        let player_hand = game.show("player 1 hand");

        assert_eq!(player_hand, "".to_string());
    }

    #[test]
    fn it_can_apply_a_winner() {
        let body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "winner".to_string(),
                    arguments: vec!(Expression::Number(1.0))
                }
            )
        ];

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

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
        let body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "winner".to_string(),
                    arguments: vec!(Expression::Symbol("player:id".to_string()))
                }
            )
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!["player".to_string()], name, body };
        let statement = Statement::Definition(definition);
        let ast = vec![declaration, statement];

        let mut game = Game::new(ast);
        game.start();
        game.player_move(1).unwrap();

        let display = game.show("game");

        assert_eq!(display, "active\nwinners: 1");
    }

    #[test]
    fn it_can_show_a_winner_after_game_over() {
        let body = vec![
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
        ];

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over\nwinners: 1");
    }

    #[test]
    fn it_executes_if_statement_when_expression_is_true() {
        let if_body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let if_statement = IfStatement{ else_body: vec![],
            expression: Expression::Bool(true),
            body: if_body
        };

        let body = vec![
            Statement::IfStatement(if_statement)
        ];

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over");
    }

    #[test]
    fn it_doesnt_execute_if_statement_when_expression_is_false() {
        let if_body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let if_statement = IfStatement{ else_body: vec![],
            expression: Expression::Bool(false),
            body: if_body
        };

        let body = vec![
            Statement::IfStatement(if_statement)
        ];

        let name = "setup".to_owned();
        let definition = Definition{ arguments: vec![], name, body };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "active");
    }

    #[test]
    fn it_executes_if_statement_when_expression_is_true_comparison() {
        let if_body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let comparison = Comparison{
            left: Expression::Number(1.0),
            right: Expression::Number(1.0),
            negative: false
        };

        let if_statement = IfStatement{ else_body: vec![],
            expression: Expression::Comparison(Box::new(comparison)),
            body: if_body
        };

        let body = vec![
            Statement::IfStatement(if_statement)
        ];

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec![] };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over");
    }

    #[test]
    fn it_can_compare_based_on_function_calls() {
        let mut ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(2.0)
                }
            )
        ];
        let if_body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let count_call = FunctionCall {
            name: "count".to_string(),
            arguments: vec![
                Expression::Symbol("player:hand".to_string())
            ]
        };

        let comparison = Comparison{
            left: Expression::FunctionCall(count_call),
            right: Expression::Number(0.0),
            negative: false
        };

        let if_statement = IfStatement{ else_body: vec![],
            expression: Expression::Comparison(Box::new(comparison)),
            body: if_body
        };

        let body = vec![
            Statement::IfStatement(if_statement)
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ name, body, arguments: vec![] };
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
        let mut ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::Players,
                    value: Expression::Number(2.0)
                }
            )
        ];
        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None; //Some(TransferModifier::Alternate);
        let count = Some(TransferCount::End);
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec![transfer_statement];
        let definition = Definition{ name, body, arguments: vec![] };
        let statement = Statement::Definition(definition);

        ast.push(statement);

        let if_body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let count_call = FunctionCall {
            name: "count".to_string(),
            arguments: vec![
                Expression::Symbol("player:hand".to_string())
            ]
        };

        let comparison = Comparison{
            left: Expression::FunctionCall(count_call),
            right: Expression::Number(26.0),
            negative: false
        };

        let if_statement = IfStatement{ else_body: vec![],
            expression: Expression::Comparison(Box::new(comparison)),
            body: if_body
        };

        let body = vec![
            Statement::IfStatement(if_statement)
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ name, body, arguments: vec!["player".to_string()] };
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
        let body = vec![
            Statement::CheckStatement(CheckStatement{ message: None,
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
        ];

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec![] };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "active");
    }

    #[test]
    fn check_passes_through_when_expression_is_true() {
        let body = vec![
            Statement::CheckStatement(CheckStatement{ message: None,
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
        ];

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec![] };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

        let mut game = Game::new(ast);
        game.start();

        let display = game.show("game");

        assert_eq!(display, "game over\nwinners: 1");
    }

    #[test]
    fn it_shows_current_player() {
        let ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::CurrentPlayer,
                    value: Expression::Number(1.0)
                }
            )
        ];

        let game = Game::new(ast);
        let current_player = game.show("current_player");

        assert_eq!(current_player, "1");
    }

    #[test]
    fn it_shows_current_player_as_set() {
        let ast = vec![
            Statement::Declaration(
                Declaration {
                    key: GlobalKey::CurrentPlayer,
                    value: Expression::Number(2.0)
                }
            )
        ];

        let game = Game::new(ast);
        let current_player = game.show("current_player");

        assert_eq!(current_player, "2");
    }

    #[test]
    fn it_can_rotate_current_player() {
        let body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "next_player".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let name = "setup".to_owned();
        let definition = Definition{ name, body,  arguments: vec![], };
        let statement = Statement::Definition(definition);
        let ast = vec![
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
        ];

        let mut game = Game::new(ast);
        game.start();

        let current_player = game.show("current_player");
        assert_eq!(current_player, "2");
    }

    #[test]
    fn it_can_rotate_current_player_back_to_first() {
        let body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "next_player".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec![] };
        let statement = Statement::Definition(definition);
        let ast = vec![
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
        ];

        let mut game = Game::new(ast);
        game.start();

        let current_player = game.show("current_player");
        assert_eq!(current_player, "1");
    }

    #[test]
    fn it_executes_if_statement_when_expression_is_true_and_true() {
        let if_body = vec![
            Statement::FunctionCall(
                FunctionCall{
                    name: "end".to_string(),
                    arguments: vec!()
                }
            )
        ];

        let and = And{
            left: Expression::Bool(true),
            right: Expression::Bool(true)
        };

        let if_statement = IfStatement{ else_body: vec![],
            expression: Expression::And(Box::new(and)),
            body: if_body
        };

        let body = vec![
            Statement::IfStatement(if_statement)
        ];

        let name = "setup".to_owned();
        let definition = Definition{ name, body, arguments: vec![] };
        let statement = Statement::Definition(definition);
        let ast = vec![statement];

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

        let body = vec![
            Statement::Transfer(
                Transfer{
                    from: "deck".to_string(),
                    to: "pl:hand".to_string(),
                    modifier: None,
                    count: None
                }
            )
        ];

        let name = "player_move".to_owned();
        let definition = Definition{ arguments: vec!["pl".to_string()], name, body };
        let statement = Statement::Definition(definition);

        let ast = vec![
            players,
            statement
        ];

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
        assert_eq!(
            game.player_move(1),
            Err("not allowed: check failed in player_move: has_no_cards(player) is true".to_string())
        );

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

    const MOVES_GAME: &str = "
players 2
stack middle
stack collection per player
values { Ten 9 }

def setup(){
    deck > middle 4
    deck > players 3
}

def player_move(player, move){
    valid_moves(take, drop)
}

def take(player, move){
    check(count(move:cards) is 1)
    middle > player:collection taking move:target
    check(sum(move:target) is sum(move:cards))
    player:hand > player:collection taking move:cards
}

def drop(player, move){
    check(count(move:cards) is 1)
    player:hand > middle taking move:cards
}
";

    fn play(game: &mut Game, line: &str) -> Result<(), String> {
        let input = crate::move_input::parse_move(line)?;
        game.make_move(input)
    }

    #[test]
    fn a_named_move_runs_that_move() {
        let mut game = game_from_source(MOVES_GAME);
        game.start();
        // middle: king, queen, jack, ten of diamonds; player 1: nine, seven, five
        assert_eq!(game.show("player 1"), "nine diamonds, seven diamonds, five diamonds");

        play(&mut game, "take 9d / 10d").unwrap();

        assert_eq!(game.show("player 1 collection"), "ten diamonds, nine diamonds");
        assert_eq!(game.show("middle"), "king diamonds, queen diamonds, jack diamonds");
    }

    #[test]
    fn a_failed_move_changes_nothing() {
        let mut game = game_from_source(MOVES_GAME);
        game.start();

        // the target is moved before the sum check fails, so it must be put back
        assert_eq!(
            play(&mut game, "take 9d / jd"),
            Err("not allowed: check failed in take: sum(move:target) is sum(move:cards)".to_string())
        );

        assert_eq!(game.show("player 1 collection"), "");
        assert_eq!(game.show("middle"), "king diamonds, queen diamonds, jack diamonds, ten diamonds");
        assert_eq!(game.show("player 1"), "nine diamonds, seven diamonds, five diamonds");
    }

    #[test]
    fn without_a_name_the_first_valid_move_is_used() {
        let mut game = game_from_source(MOVES_GAME);
        game.start();

        // take works (ten is worth 9), so it is used rather than drop
        play(&mut game, "9d / 10d").unwrap();
        assert_eq!(game.show("player 1 collection"), "ten diamonds, nine diamonds");

        // take fails (jack is 11, not 7) and is undone, so the card is dropped
        play(&mut game, "7d / jd").unwrap();
        assert_eq!(game.show("player 1 collection"), "ten diamonds, nine diamonds");
        assert_eq!(game.show("middle"), "king diamonds, queen diamonds, jack diamonds, seven diamonds");
        assert_eq!(game.show("player 1"), "five diamonds");
    }

    #[test]
    fn an_unknown_move_name_is_rejected() {
        let mut game = game_from_source(MOVES_GAME);
        game.start();

        assert_eq!(
            play(&mut game, "steal 9d"),
            Err("not allowed: unknown move 'steal' - valid moves: take, drop".to_string())
        );
        assert_eq!(game.show("player 1"), "nine diamonds, seven diamonds, five diamonds");
    }

    #[test]
    fn moves_can_be_made_for_another_player() {
        let mut game = game_from_source(MOVES_GAME);
        game.start();

        play(&mut game, "2 drop 8d").unwrap();

        assert_eq!(game.show("player 2"), "six diamonds, four diamonds");
    }

    #[test]
    fn cli_command_names_cannot_be_function_names() {
        for name in ["build", "start", "show", "exit", "help"].iter() {
            let source = format!("def {}(){{ end() }}", name);
            let tokens = crate::lex::lexer(&source).unwrap();
            assert!(crate::parse::parse(&tokens).is_err(), "{} should be reserved", name);
        }
    }

    fn cards_shown(game: &Game, what: &str) -> Vec<String> {
        game.show(what).split(", ").filter(|c| !c.is_empty()).map(|c| c.to_string()).collect()
    }

    // Plays a whole game of Simple Scopa with `players` players: capture a
    // single card of the same value if possible, otherwise drop.
    fn play_simple_scopa(players: usize) {
        let source = include_str!("../examples/simple_scopa.card")
            .replace("players 2", &format!("players {}", players));
        let mut game = game_from_source(&source);
        game.start();
        assert_eq!(cards_shown(&game, "middle").len(), 4);
        for n in 1..=players {
            assert_eq!(cards_shown(&game, &format!("player {}", n)).len(), 3);
        }

        let mut moves = 0;
        let mut captures = 0;
        while !game.show("game").starts_with("game over") {
            moves += 1;
            assert!(moves <= 52, "the game should end after 48 cards are played");

            let current: usize = game.show("current_player").parse().unwrap();
            let hand = cards_shown(&game, &format!("player {}", current));
            let middle = cards_shown(&game, "middle");

            // moving out of turn is rejected, with the game's message
            let other = current % players + 1;
            if let Some(card) = cards_shown(&game, &format!("player {}", other)).first() {
                assert_eq!(
                    play(&mut game, &format!("{} drop {}", other, card)),
                    Err("it's not your turn".to_string())
                );
            }

            let mut played = false;
            'search: for card in hand.iter() {
                for target in middle.iter() {
                    if play(&mut game, &format!("take {} / {}", card, target)).is_ok() {
                        captures += 1;
                        played = true;
                        break 'search;
                    }
                }
            }
            if !played {
                play(&mut game, &format!("drop {}", hand[0])).unwrap();
            }
        }

        // all 48 cards dealt to players are played, however many players there are
        assert_eq!(moves, 48);
        let collected: Vec<usize> = (1..=players)
            .map(|n| cards_shown(&game, &format!("player {} collection", n)).len())
            .collect();
        let total: usize = collected.iter().sum();
        // every card is accounted for: each capture collects two cards
        assert_eq!(total, captures * 2);
        assert_eq!(total + cards_shown(&game, "middle").len(), 52);
        assert_eq!(cards_shown(&game, "deck").len(), 0);

        // everyone with the most cards wins
        let most = *collected.iter().max().unwrap();
        let winners: Vec<String> = (1..=players)
            .filter(|n| collected[n - 1] == most)
            .map(|n| n.to_string())
            .collect();
        assert_eq!(game.show("game"), format!("game over\nwinners: {}", winners.join(", ")));
    }

    #[test]
    fn simple_scopa_can_be_played_to_the_end() {
        play_simple_scopa(2);
    }

    #[test]
    fn simple_scopa_works_for_three_and_four_players() {
        play_simple_scopa(3);
        play_simple_scopa(4);
    }

    #[test]
    fn simple_scopa_explains_mistakes() {
        // without the shuffle: middle K Q J 10 of diamonds; player 1 has 9 7 5 of diamonds
        let source = include_str!("../examples/simple_scopa.card").replace("    shuffle(deck)\n", "");
        let mut game = game_from_source(&source);
        game.start();

        assert_eq!(play(&mut game, "take 9d 10d"), Err("play one card from your hand, e.g. take 9d / 4c 5s".to_string()));
        assert_eq!(play(&mut game, "take 9d"), Err("put a / before the cards you're capturing, e.g. take 9d / 4c 5s".to_string()));
        assert_eq!(play(&mut game, "take 8d / 10d"), Err("you can only play a card from your hand".to_string()));
        assert_eq!(play(&mut game, "take 9d / 3c"), Err("you can only capture cards from the middle".to_string()));
        assert_eq!(play(&mut game, "take 9d / 10d"), Err("the cards you capture must add up to the card you play".to_string()));
        assert_eq!(play(&mut game, "drop 9d / 10d"), Err("to capture cards use take, e.g. take 9d / 4c 5s".to_string()));
        assert_eq!(play(&mut game, "2 drop 8d"), Err("it's not your turn".to_string()));
        play(&mut game, "drop 9d").unwrap();
    }

    #[test]
    fn simple_scopa_take_needs_matching_values() {
        let mut game = game_from_source(include_str!("../examples/simple_scopa.card"));
        game.start();

        let hand = cards_shown(&game, "player 1");
        let middle = cards_shown(&game, "middle");

        // a drop with a target, or a take with no target, is never valid
        assert!(play(&mut game, &format!("drop {} / {}", hand[0], middle[0])).is_err());
        assert!(play(&mut game, &format!("take {}", hand[0])).is_err());
        // cards that aren't in your hand can't be played
        let not_mine = cards_shown(&game, "player 2")[0].clone();
        assert!(play(&mut game, &format!("drop {}", not_mine)).is_err());

        assert_eq!(cards_shown(&game, "player 1"), hand);
        assert_eq!(cards_shown(&game, "middle"), middle);
    }

    #[test]
    fn a_rejected_move_without_a_name_explains_every_attempt() {
        let mut game = game_from_source(MOVES_GAME);
        game.start();

        // two cards: take needs one played card, and so does drop
        assert_eq!(
            play(&mut game, "9d 7d"),
            Err("not allowed: no valid move - check failed in take: count(move:cards) is 1; \
                 check failed in drop: count(move:cards) is 1".to_string())
        );
    }

    #[test]
    fn a_failed_transfer_explains_which_card_is_missing() {
        let mut game = game_from_source(MOVES_GAME);
        game.start();

        assert_eq!(
            play(&mut game, "drop kc"),
            Err("not allowed: transfer failed in drop: player:hand > middle taking move:cards \
                 (king clubs is not in player:hand)".to_string())
        );
    }

    #[test]
    fn a_check_message_is_shown_instead_of_the_rule() {
        let mut game = game_from_source("
players 2
stack middle

def setup(){
    deck > players 3
}

def player_move(player, move){
    check(count(move:target) > 0, put a / before the cards you're capturing, e.g. take 7h / 7c)
    end()
}
");
        game.start();

        assert_eq!(
            play(&mut game, "take 9d"),
            Err("put a / before the cards you're capturing, e.g. take 7h / 7c".to_string())
        );
        assert_eq!(game.show("game"), "active");
    }

    #[test]
    fn check_messages_without_a_name_are_listed_with_the_other_reasons() {
        let mut game = game_from_source("
players 2

def player_move(player, move){
    valid_moves(take, drop)
}

def take(player, move){
    check(count(move:target) > 0, nothing to take)
}

def drop(player, move){
    check(count(move:cards) is 0)
}
");
        game.start();

        assert_eq!(
            play(&mut game, "9d"),
            Err("not allowed: no valid move - nothing to take; check failed in drop: count(move:cards) is 0".to_string())
        );
    }

    #[test]
    fn every_player_runs_a_block_for_each_player_in_order() {
        let mut game = game_from_source("
players 3

def setup(){
    every player p {
        deck > p:hand 2
        winner(p:id)
    }
}
");
        game.start();

        assert_eq!(game.show("game"), "active\nwinners: 1, 2, 3");
        assert_eq!(game.show("players"), "player 1 (cards: 2), player 2 (cards: 2), player 3 (cards: 2)");
        assert_eq!(game.show("player 1"), "king diamonds, queen diamonds");
    }

    #[test]
    fn every_player_as_a_test_is_true_only_if_it_holds_for_all() {
        let result = winners_of("
players 3

def setup(){
    if(every player p: count(p:hand) is 0){
        winner(1)
    }
    deck > players:2:hand
    if(every player p: count(p:hand) is 0){
        winner(2)
    }
    if(every player p : p:id >= 1 & p:id <= 3){
        winner(3)
    }
}
");
        assert_eq!(result, "active\nwinners: 1, 3");
    }

    #[test]
    fn every_card_goes_through_the_cards_in_a_pile() {
        let result = winners_of("
players 2
stack middle

def setup(){
    deck > middle 3
    every card c in middle {
        winner(c:value)
    }
    if(every card c in middle: c:suit is Diamonds){
        winner(100)
    }
    if(every card c in middle : c:value > 11){
        winner(200)
    }
}
");
        // king, queen, jack of diamonds
        assert_eq!(result, "active\nwinners: 13, 12, 11, 100");
    }

    #[test]
    fn every_card_works_with_player_piles() {
        let mut game = game_from_source("
players 2

def setup(){
    deck > players 2
}

def player_move(player){
    check(every card c in player:hand: c:suit is Diamonds, all diamonds)
    end()
}
");
        game.start();
        game.player_move(1).unwrap();
        assert_eq!(game.show("game"), "game over");
    }

    #[test]
    fn return_and_check_inside_every_end_the_function() {
        let result = winners_of("
players 3

def first_with_id_above(n){
    every player p {
        if(p:id > n){
            return(p:id)
        }
    }
    return(0)
}

def setup(){
    winner(first_with_id_above(1))
    winner(first_with_id_above(5))
    every player p {
        check(p:id < 2)
        winner(10)
    }
    winner(99)
}
");
        assert_eq!(result, "active\nwinners: 2, 0, 10");
    }

    #[test]
    fn players_with_the_most_cards_can_be_found_without_variables() {
        let result = winners_of("
players 3
stack collection per player

def setup(){
    deck > players:1:collection 2
    deck > players:2:collection 5
    deck > players:3:collection 5
    every player p {
        if(every player q: count(p:collection) >= count(q:collection)){
            winner(p:id)
        }
    }
}
");
        assert_eq!(result, "active\nwinners: 2, 3");
    }

    #[test]
    fn a_deck_can_leave_out_ranks() {
        let result = winners_of("
players 2
deck standard without Eight Nine Ten

def is_eight(card){
    return(card:rank is Eight or card:rank is Ten)
}

def setup(){
    winner(count(deck))
    winner(count(filter(deck, is_eight)))
}
");
        assert_eq!(result, "active\nwinners: 40, 0");
    }

    #[test]
    fn a_named_values_table_becomes_a_card_field() {
        let result = winners_of("
players 2
values { King 10 }
values primiera {
    Seven 21, Six 18
}

def is_seven(card){
    return(card:primiera is 21 & card:value is 7)
}

def is_king(card){
    return(card:rank is King & card:primiera is 0 & card:value is 10)
}

def setup(){
    winner(count(filter(deck, is_seven)))
    winner(count(filter(deck, is_king)))
}
");
        assert_eq!(result, "active\nwinners: 4, 4");
    }

    #[test]
    fn card_literals_are_piles_of_specific_cards() {
        let mut game = game_from_source("
players 2
stack middle

def setup(){
    deck > middle taking [7d, kh]
    if(cards_in_stack([7d], middle) & count([as, 2c, 3h]) is 3){
        winner(1)
    }
    if(cards_in_stack([7d], deck)){
        winner(2)
    }
}
");
        game.start();
        assert_eq!(game.show("middle"), "seven diamonds, king hearts");
        assert_eq!(game.show("game"), "active\nwinners: 1");
    }

    #[test]
    fn a_line_break_inside_brackets_continues_the_statement() {
        let result = winners_of("
players 2

def add(a, b, c){
    return(a + b
        + c)
}

def setup(){
    winner(add(1,
               2,
               3))
    winner(4)
}
");
        assert_eq!(result, "active\nwinners: 6, 4");
    }

    #[test]
    fn any_is_true_when_at_least_one_matches() {
        let result = winners_of("
players 3
stack middle

def setup(){
    if(any player p: count(p:hand) > 0){
        winner(1)
    }
    deck > players:2:hand
    if(any player p: count(p:hand) > 0){
        winner(2)
    }
    if(any card c in middle: true){
        winner(3)
    }
    if(any card c in [7d, kh]: c:rank is King){
        winner(4)
    }
}
");
        // any over an empty pile is false
        assert_eq!(result, "active\nwinners: 2, 4");
    }

    #[test]
    fn any_group_tries_every_combination_of_cards() {
        let result = winners_of("
players 2

def setup(){
    .(3 + 7 is 10, but no combination of 3, 7 and 13 makes 12)
    if(any group g in [3c, 7d, kh]: sum(g) is 10){
        winner(1)
    }
    if(any group g in [3c, 7d, kh]: sum(g) is 12){
        winner(2)
    }
    if(every group g in [3c, 7d]: count(g) >= 1){
        winner(3)
    }
    every group g in [as, 2c] {
        winner(sum(g))
    }
}
");
        assert_eq!(result, "active\nwinners: 1, 3, 1, 2, 3");
    }

    #[test]
    fn group_refuses_piles_that_are_too_big() {
        let result = winners_of("
players 2

def setup(){
    if(any group g in deck: count(g) is 1){
        winner(1)
    }
    winner(2)
}
");
        // 52 cards would be 2^52 combinations
        assert_eq!(result, "active\nwinners: 2");
    }
}
