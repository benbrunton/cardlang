//! cardlang: a language for the rules of card games, and the engine that
//! runs them.
//!
//! The CLI (`src/main.rs`) is one program built on this library; a server or
//! a WebAssembly build can use it the same way:
//!
//! ```
//! let source = "name demo\nplayers 2\ndef setup(){\n    shuffle(deck)\n    deck > players 5\n}";
//! let mut game = cardlang::Game::from_source(source).unwrap();
//! game.start_with_seed(42);
//! assert_eq!(game.show("current_player"), "1");
//! ```

mod ast;
mod cards;
mod interpreter;
mod lex;
mod move_input;
mod parse;
mod runtime;
mod token;

pub use cards::{card_code, Card};
pub use interpreter::{CompletionContext, Game};
pub use move_input::{parse_move, RESERVED_WORDS};
pub use runtime::MoveInput;
