use std::io::{stdin, stdout, Write};
use std::fs;

mod lex;
mod token;

struct Game;

enum CommandResult {
    Game(Game),
    CommandFailed,
    Exit
}

fn main() {
    println!("Cardlang interpreter");
    let mut game;

    loop {
        print!("> ");
        let _ = stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let command = input.trim().split(' ').collect();

        let command_result = handle_command(command);
        
        match command_result {
            CommandResult::Exit => break,
            CommandResult::Game(g) => game = g,
            _ => ()
        }
    }
}

fn handle_command(command: Vec<&str>) -> CommandResult {
    match command[0] {
        "exit" => CommandResult::Exit,
        "build" => build_game(command),
        _ => unrecognised_command()
    }
}

fn build_game(command: Vec<&str>) -> CommandResult {
    if command.len() < 2 {
        println!("no source file specified in build");
        return CommandResult::CommandFailed;
    }

    let file_result = fs::read_to_string(command[1]);

    if file_result.is_err() {
        println!("unable to read '{}'", command[1]);
        return CommandResult::CommandFailed;
    }

    let game = parse_game(file_result.expect("unable to read file"));
    CommandResult::Game(game)
}

fn parse_game(source: String) -> Game {
    let tokens = lex::lexer(&source);

    println!("Game loaded");
    Game
}

fn unrecognised_command() -> CommandResult {
    println!("unrecognised command");
    CommandResult::CommandFailed
}