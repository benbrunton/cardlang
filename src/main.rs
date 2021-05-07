use std::io::{stdin, stdout, Write};
use std::fs;

mod lex;
mod parse;
mod token;
mod ast;
mod interpreter;
mod cards;
mod runtime;

use interpreter::Game;

enum CommandResult {
    Game(Game),
    CommandFailed,
    Exit,
    Show(String),
    Start,
    Move(usize)
}

fn main() {
    println!("Cardlang interpreter");
    let mut game: Option<Game> = None;

    loop {
        print!("> ");
        let _ = stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let command = input.trim().split(' ').collect();
        let command_result = translate_command(command);

        // handle global commands
        match command_result {
            CommandResult::Game(ref g) => game = Some(g.clone()),
            CommandResult::Exit => break,
            _ => ()
        }

        handle_game_command(command_result, &mut game);
    }
}

fn translate_command(command: Vec<&str>) -> CommandResult {
    match command[0] {
        "exit" => CommandResult::Exit,
        "build" => build_game(command),
        "show" => {
            let display_list = &command[1..];
            CommandResult::Show(display_list.join(" "))
        },
        "start" => CommandResult::Start,
        "move" => CommandResult::Move(command[1].parse().unwrap_or(1)),
        _ => unrecognised_command()
    }
}

fn handle_game_command(command: CommandResult, game: &mut Option<Game>) {
    if let Some(ref mut g) = game { 
        match command {
            CommandResult::Show(c) => println!("{}", g.show(&c)),
            CommandResult::Start => g.start(),
            CommandResult::Move(n) => g.player_move(n),
            _ => ()
        }
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

    match game {
        Some(g) => CommandResult::Game(g),
        None => CommandResult::CommandFailed 
    }
}

fn parse_game(source: String) -> Option<Game> {
    let lex_result = lex::lexer(&source);
    if lex_result.is_err() {
        println!("parse error: {:?}", lex_result.unwrap_err());
        return None;
    }

    let tokens = lex_result.expect("unable to unwrap tokens");

    let parse_result = parse::parse(tokens);

    if parse_result.is_err() {
        println!("parse error: {:?}", parse_result.unwrap_err());
        return None;
    }

    let ast = parse_result.expect("unable to unwrap ast!");
    let game = Game::new(ast);
    println!("Game loaded");
    Some(game)
}

fn unrecognised_command() -> CommandResult {
    println!("unrecognised command");
    CommandResult::CommandFailed
}