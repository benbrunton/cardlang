// Tab completion for the interactive CLI.
//
// `complete` decides what can follow the text before the cursor. It is a
// plain function so it can be tested without a terminal; `CliHelper` plugs
// it into rustyline and adds file name completion for `build`.
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

use crate::interpreter::CompletionContext;
use crate::move_input::RESERVED_WORDS;

const SHOW_TARGETS: [&str; 6] = ["game", "name", "players", "current_player", "deck", "player"];

#[derive(Debug, PartialEq)]
pub enum Completion {
    // replace the text from this position with one of these words
    Words(usize, Vec<String>),
    // complete a file or folder name (after `build`)
    Path
}

pub fn complete(line: &str, pos: usize, context: Option<&CompletionContext>) -> Completion {
    let before = &line[..pos];

    // A path after `build` can contain '/', so check for it before splitting on '/'.
    // The cursor is in the word after `build` if that's the second word being typed.
    let whole_words: Vec<&str> = before.split_whitespace().collect();
    let starting_new_word = before.is_empty() || before.ends_with(char::is_whitespace);
    let word_index = if starting_new_word { whole_words.len() } else { whole_words.len() - 1 };
    if whole_words.first() == Some(&"build") && word_index == 1 {
        return Completion::Path;
    }

    // the word being typed starts after the last space, '/' or ','
    let start = before.rfind(|c: char| c.is_whitespace() || c == '/' || c == ',').map_or(0, |i| i + 1);
    let word = &before[start..];
    let previous: Vec<&str> = before[..start].split_whitespace().collect();

    let candidates: Vec<String> = match previous.first().copied() {
        None => {
            let mut words: Vec<String> = RESERVED_WORDS.iter().map(|w| w.to_string()).collect();
            if let Some(c) = context {
                words.extend(c.move_names.iter().cloned());
            }
            words
        },
        Some("show") => show_candidates(&previous[1..], context),
        Some(w) if RESERVED_WORDS.contains(&w) => vec![],
        Some(_) => match context {
            Some(c) => move_candidates(before, start, &previous, c),
            None => vec![]
        }
    };

    let lowered = word.to_lowercase();
    let matches = candidates.into_iter()
        .filter(|c| c.to_lowercase().starts_with(&lowered))
        .collect();
    Completion::Words(start, matches)
}

fn show_candidates(arguments: &[&str], context: Option<&CompletionContext>) -> Vec<String> {
    let empty = CompletionContext::default();
    let context = context.unwrap_or(&empty);
    match arguments {
        [] => SHOW_TARGETS.iter().map(|s| s.to_string())
            .chain(context.stacks.iter().cloned())
            .collect(),
        ["player"] => (1..=context.players).map(|n| n.to_string()).collect(),
        ["player", _] => std::iter::once("hand".to_string())
            .chain(context.player_stacks.iter().cloned())
            .collect(),
        _ => vec![]
    }
}

// a move is `[player] [name] [cards] [/ cards]`
fn move_candidates(before: &str, start: usize, previous: &[&str], context: &CompletionContext) -> Vec<String> {
    let words: Vec<&str> = match previous.first() {
        Some(p) if p.chars().all(|c| c.is_ascii_digit()) => previous[1..].to_vec(),
        _ => previous.to_vec()
    };

    // after a player number, the next word can be a move name
    let mut candidates = vec![];
    if words.is_empty() {
        candidates.extend(context.move_names.iter().cloned());
    }

    // cards from the hand, or from the table after a '/'
    let typed: Vec<String> = before[..start]
        .split(|c: char| c.is_whitespace() || c == '/' || c == ',')
        .map(|w| w.to_lowercase())
        .collect();
    let cards = if before[..start].contains('/') { &context.table } else { &context.hand };
    candidates.extend(cards.iter().filter(|c| !typed.contains(c)).cloned());
    candidates
}

pub struct CliHelper {
    pub context: Option<CompletionContext>,
    files: FilenameCompleter
}

impl CliHelper {
    pub fn new() -> CliHelper {
        CliHelper { context: None, files: FilenameCompleter::new() }
    }
}

impl Completer for CliHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        match complete(line, pos, self.context.as_ref()) {
            Completion::Words(start, words) => {
                let pairs = words.into_iter()
                    .map(|w| Pair { display: w.clone(), replacement: format!("{} ", w) })
                    .collect();
                Ok((start, pairs))
            },
            Completion::Path => {
                // only offer folders and game files
                let (start, candidates) = self.files.complete(line, pos, ctx)?;
                let games = candidates.into_iter()
                    .filter(|p| p.replacement.ends_with('/') || p.replacement.ends_with(".card"))
                    .collect();
                Ok((start, games))
            }
        }
    }
}

impl Hinter for CliHelper {
    type Hint = String;
}

impl Highlighter for CliHelper {}

impl Validator for CliHelper {}

impl Helper for CliHelper {}

#[cfg(test)]
mod test {
    use super::*;

    fn scopa_context() -> CompletionContext {
        CompletionContext {
            move_names: vec!["drop".to_string(), "take".to_string()],
            stacks: vec!["middle".to_string()],
            player_stacks: vec!["collection".to_string()],
            players: 2,
            hand: vec!["7h".to_string(), "10d".to_string(), "ks".to_string()],
            table: vec!["3c".to_string(), "4s".to_string(), "7c".to_string()]
        }
    }

    fn words(line: &str, context: Option<&CompletionContext>) -> Vec<String> {
        match complete(line, line.len(), context) {
            Completion::Words(_, w) => w,
            Completion::Path => panic!("expected words, got a path completion")
        }
    }

    #[test]
    fn the_first_word_can_be_a_command_or_a_move() {
        let context = scopa_context();
        assert_eq!(words("b", Some(&context)), vec!("build"));
        assert_eq!(words("s", Some(&context)), vec!("start", "show"));
        assert_eq!(words("t", Some(&context)), vec!("take"));
        assert_eq!(words("", None), vec!("build", "start", "show", "exit", "help"));
    }

    #[test]
    fn build_completes_file_names() {
        assert_eq!(complete("build ex", 8, None), Completion::Path);
        assert_eq!(complete("build ", 6, None), Completion::Path);
        assert_eq!(complete("build examples/si", 17, None), Completion::Path);
        // only one file name is completed
        assert_eq!(complete("build a.card ", 13, None), Completion::Words(13, vec!()));
    }

    #[test]
    fn show_completes_what_can_be_shown() {
        let context = scopa_context();
        assert_eq!(words("show m", Some(&context)), vec!("middle"));
        assert_eq!(words("show p", Some(&context)), vec!("players", "player"));
        assert_eq!(words("show player ", Some(&context)), vec!("1", "2"));
        assert_eq!(words("show player 1 c", Some(&context)), vec!("collection"));
    }

    #[test]
    fn moves_complete_cards_from_the_hand_then_the_table() {
        let context = scopa_context();
        assert_eq!(words("take ", Some(&context)), vec!("7h", "10d", "ks"));
        assert_eq!(words("take 1", Some(&context)), vec!("10d"));
        assert_eq!(words("take 7h / ", Some(&context)), vec!("3c", "4s", "7c"));
        assert_eq!(words("take 7h / 3c ", Some(&context)), vec!("4s", "7c"));
        assert_eq!(words("take 7h/7", Some(&context)), vec!("7c"));
    }

    #[test]
    fn a_player_number_can_come_first() {
        let context = scopa_context();
        assert_eq!(words("2 d", Some(&context)), vec!("drop"));
        assert_eq!(words("2 drop K", Some(&context)), vec!("ks"));
    }

    #[test]
    fn the_cursor_position_decides_what_is_completed() {
        let context = scopa_context();
        // cursor after "ta", with more text after it
        assert_eq!(complete("ta 7h", 2, Some(&context)), Completion::Words(0, vec!("take".to_string())));
    }

    #[test]
    fn commands_without_arguments_complete_nothing() {
        let context = scopa_context();
        assert!(words("start ", Some(&context)).is_empty());
        assert!(words("exit ", Some(&context)).is_empty());
    }
}

