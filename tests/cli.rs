use std::io::Write;
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_cardlang");

fn run_interactive(input: &str) -> Output {
    let mut child = Command::new(BIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("unable to start cardlang");

    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();

    // if end of input is not handled this would hang, so poll with a timeout
    let started = std::time::Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if started.elapsed().as_secs() > 10 {
            child.kill().unwrap();
            panic!("cardlang did not exit at end of input");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.wait_with_output().unwrap()
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_command_succeeds_for_a_valid_file() {
    let output = Command::new(BIN).args(["test", "examples/race.card"]).output().unwrap();
    assert!(output.status.success());
    assert!(stdout_of(&output).contains("success!"));
}

#[test]
fn test_command_fails_for_an_invalid_file() {
    let output = Command::new(BIN).args(["test", "examples/invalid_spec.card"]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_command_fails_for_a_missing_file() {
    let output = Command::new(BIN).args(["test", "examples/does_not_exist.card"]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_command_fails_without_a_file() {
    let output = Command::new(BIN).arg("test").output().unwrap();
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn interactive_mode_exits_at_end_of_input() {
    let output = run_interactive("show game\n");
    assert!(output.status.success());
}

#[test]
fn interactive_mode_ignores_empty_lines() {
    let output = run_interactive("\n\nexit\n");
    assert!(!stdout_of(&output).contains("unrecognised command"));
}

#[test]
fn commands_before_build_explain_what_to_do() {
    let output = run_interactive("start\nexit\n");
    assert!(stdout_of(&output).contains("no game loaded"));
}

#[test]
fn bad_moves_are_reported_and_do_not_crash() {
    let output = run_interactive(
        "build ./examples/race.card\nplay\nstart\nplay 2\n0 play\n3 play\ntake 7x\nshow player\nshow player 9\n2 play\nplay\nshow current_player\nexit\n"
    );
    let stdout = stdout_of(&output);

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains("move ignored: the game has not started"));
    assert!(stdout.contains("to move as player 2, put the number first: '2 play'"));
    assert!(stdout.contains("move ignored: there is no player 0"));
    assert!(stdout.contains("move ignored: there is no player 3"));
    assert!(stdout.contains("'7x' is not a card"));
    assert!(stdout.contains("usage: show player <number>"));
    assert!(stdout.contains("there is no player 9"));
    // player 2 can't move out of turn...
    assert!(stdout.contains("move ignored: it's not your turn"));
    // ...but the current player can, so it's now player 2's turn
    assert!(stdout.contains("> 2\n"));
}

#[test]
fn moving_in_a_game_without_moves_is_reported() {
    let dir = std::env::temp_dir().join(format!("cardlang-cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("no_moves.card");
    std::fs::write(&file, "name no_moves\nplayers 2\n").unwrap();

    let output = run_interactive(&format!("build {}\nstart\nplay\nexit\n", file.display()));
    std::fs::remove_dir_all(&dir).ok();

    assert!(output.status.success());
    assert!(stdout_of(&output).contains("move ignored: this game has no moves - declare them with 'move'"));
}

#[test]
fn help_lists_the_commands() {
    let output = run_interactive("help\nexit\n");
    let stdout = stdout_of(&output);
    assert!(stdout.contains("build <file>"));
    assert!(stdout.contains("take 7h / 3c 4s"));
}

#[test]
fn show_before_start_says_to_start() {
    let output = run_interactive("build ./examples/simple_scopa.card\nshow middle\nshow player 1\nshow game\nexit\n");
    let stdout = stdout_of(&output);
    assert_eq!(stdout.matches("the game hasn't started - type 'start' to deal").count(), 2);
    assert!(stdout.contains("pending"));
}

#[test]
fn the_table_is_shown_after_start_and_after_each_move() {
    let output = run_interactive("build ./examples/simple_scopa.card\nstart\nexit\n");
    let stdout = stdout_of(&output);
    assert!(stdout.contains("player 1 to play"));
    assert!(stdout.contains("  deck: 42 cards"));
    assert!(stdout.contains("  middle: "));
    assert!(stdout.contains("  player 1: collection 0 cards"));
    assert!(stdout.contains("  player 2: collection 0 cards"));
    assert!(stdout.contains("  your hand (player 1): "));
}

#[test]
fn the_table_after_a_move_shows_the_next_player() {
    // Simple Scopa without the shuffle, so the deal is always the same:
    // middle king/queen/jack/ten of diamonds, player 1 nine/seven/five of diamonds
    let source = std::fs::read_to_string("examples/simple_scopa.card").unwrap()
        .replace("    shuffle(deck)\n", "");
    let dir = std::env::temp_dir().join(format!("cardlang-cli-table-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("unshuffled_scopa.card");
    std::fs::write(&file, source).unwrap();

    let output = run_interactive(&format!("build {}\nstart\ndrop 9d\nexit\n", file.display()));
    std::fs::remove_dir_all(&dir).ok();
    let stdout = stdout_of(&output);

    assert!(stdout.contains("  your hand (player 1): 9d, 7d, 5d"));
    assert!(stdout.contains("player 2 to play"));
    assert!(stdout.contains("  middle: kd, qd, jd, 10d, 9d"));
    assert!(stdout.contains("  your hand (player 2): 8d, 6d, 4d"));
}

#[test]
fn a_rejected_move_says_why() {
    let output = run_interactive("build ./examples/simple_scopa.card\nstart\nplay 7h\nexit\n");
    assert!(stdout_of(&output).contains("move ignored: unknown move 'play' - the moves are: take, drop"));
}

fn run_with_args(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(BIN)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("unable to start cardlang");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn play_builds_and_starts_a_game() {
    let output = run_with_args(&["play", "examples/simple_scopa.card"], "show game\nexit\n");
    let stdout = stdout_of(&output);
    assert!(output.status.success());
    assert!(stdout.contains("> build examples/simple_scopa.card\nGame loaded"));
    assert!(stdout.contains("> start\nplayer 1 to play"));
    assert!(stdout.contains("active"));
}

#[test]
fn play_without_a_file_is_an_error() {
    let output = run_with_args(&["play"], "");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage: cardlang play <file>"));
}
