// The engine used as a library, the way a server or WebAssembly build would:
// only through the public API, with no CLI.
use std::time::{Duration, Instant};

use cardlang::{parse_move, Game};

const SCOPA: &str = include_str!("../examples/scopa.card");
const SIMPLE_SCOPA: &str = include_str!("../examples/simple_scopa.card");

fn started(source: &str, seed: u64) -> Game {
    let mut game = Game::from_source(source).expect("the game should load");
    game.start_with_seed(seed);
    game
}

#[test]
fn a_game_loads_from_source_and_starts() {
    let game = started(SCOPA, 1);
    assert!(game.is_started());
    assert_eq!(game.show("name"), "scopa");
}

#[test]
fn a_game_that_does_not_parse_says_why() {
    let error = Game::from_source("name demo\nplayers two").expect_err("should not load");
    assert!(error.contains("line 2"), "{}", error);
}

#[test]
fn the_same_seed_deals_the_same_cards() {
    let a = started(SCOPA, 42);
    let b = started(SCOPA, 42);
    assert_eq!(a.table(), b.table());
    assert_eq!(a.show("deck"), b.show("deck"));
    assert_eq!(a.seed(), 42);
}

#[test]
fn different_seeds_deal_different_cards() {
    let decks: Vec<String> = (0..5).map(|seed| started(SCOPA, seed).show("deck")).collect();
    assert!(decks.iter().any(|d| *d != decks[0]), "five seeds all dealt the same deck");
}

#[test]
fn a_seeded_deal_never_changes() {
    // Pins the shuffle itself: if this fails, saved seeds no longer replay the
    // same games (e.g. after changing the random number generator).
    let game = started("name demo\nplayers 1\ndef setup(){\n    shuffle(deck)\n}", 7);
    let deck = game.show("deck");
    let first_five: Vec<&str> = deck.split(", ").take(5).collect();
    assert_eq!(first_five, ["four diamonds", "eight diamonds", "king clubs", "three clubs", "nine hearts"]);
}

#[test]
fn a_whole_game_replays_the_same_from_its_seed() {
    // play the first legal-looking move each turn: drop the first card in hand
    fn play(seed: u64) -> Vec<String> {
        let mut game = started(SIMPLE_SCOPA, seed);
        let mut tables = vec![game.table()];
        for _ in 0..40 {
            if game.show("game") != "active" {
                break;
            }
            let hand = game.show(&format!("player {}", game.show("current_player")));
            let card = hand.split(", ").next().unwrap_or("").to_string();
            let input = parse_move(&format!("drop {}", card));
            if let Ok(input) = input {
                let _ = game.make_move(input);
            }
            tables.push(game.table());
        }
        tables
    }
    assert_eq!(play(3), play(3));
}

#[test]
fn warnings_are_returned_to_the_caller_not_printed() {
    let mut game = started("name demo\nplayers 1\ndef setup(){\n    no_such_function()\n}", 1);
    assert_eq!(game.take_messages(), vec!["unknown function 'no_such_function'".to_string()]);
    assert!(game.take_messages().is_empty(), "messages are only given once");
}

const RUNAWAY: &str = "name runaway
players 2

.(calls itself twice: 2 to the power 200 calls without a limit)
def spin(n){
    if(n > 0){
        spin(n - 1)
        spin(n - 1)
    }
}

move go(player){
    spin(200)
}

move wait(player){
    check(1 is 1)
}
";

#[test]
fn a_move_that_does_too_much_work_is_stopped_and_undone() {
    let mut game = started(RUNAWAY, 1);
    let before = game.table();
    let started_at = Instant::now();
    let error = game.make_move(parse_move("go").unwrap()).expect_err("the move should be stopped");
    assert!(started_at.elapsed() < Duration::from_secs(5), "took {:?}", started_at.elapsed());
    assert!(error.contains("too much work"), "{}", error);
    assert_eq!(game.table(), before, "a stopped move changes nothing");

    // the game carries on
    assert!(game.make_move(parse_move("wait").unwrap()).is_ok());
}

#[test]
fn setup_that_does_too_much_work_is_stopped() {
    let source = "name runaway\nplayers 1\ndef spin(n){\n    if(n > 0){\n        spin(n - 1)\n        spin(n - 1)\n    }\n}\ndef setup(){\n    spin(200)\n}";
    let mut game = started(source, 1);
    let messages = game.take_messages();
    assert!(messages.iter().any(|m| m.starts_with("setup stopped") && m.contains("too much work")), "{:?}", messages);
}

#[test]
fn the_step_limit_can_be_lowered() {
    let mut game = Game::from_source(SIMPLE_SCOPA).unwrap();
    game.set_step_limit(5);
    game.start_with_seed(1);
    let hand = game.show("player 1");
    let card = hand.split(", ").next().unwrap().to_string();
    let error = game.make_move(parse_move(&format!("drop {}", card)).unwrap()).expect_err("5 steps is too few");
    assert!(error.contains("over 5 steps"), "{}", error);
}
