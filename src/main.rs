use std::{fs, env, process, io::{stdin, stdout, Write}};

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
    Move(usize),
    Nothing
}

fn main() {

    let default_command = "".to_string();
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).unwrap_or(&default_command);
    match command as &str {
        "test"    => {
            if !test(args.get(2)) {
                process::exit(1);
            }
        },
        _         => interactive()
    }
}

// returns true when the file parses
fn test(cmd: Option<&String>) -> bool {
    match cmd {
        Some(path) => {
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("unable to read '{}'", path);
                    return false;
                }
            };

            match parse_game(source) {
                Some(_g) => {
                    println!("success!");
                    true
                },
                None => {
                    eprintln!("failed to parse!");
                    false
                }
            }
        },
        None => {
            eprintln!("no file specified!");
            false
        }
    }
}

fn interactive() {
    println!("Cardlang interpreter");
    let mut game: Option<Game> = None;
    loop {
        print!("> ");
        let _ = stdout().flush();

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            // end of input (e.g. ctrl-d or a closed pipe)
            Ok(0) | Err(_) => {
                println!();
                break;
            },
            Ok(_) => ()
        }

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
        "" => CommandResult::Nothing,
        "exit" => CommandResult::Exit,
        "build" => build_game(command),
        "show" => {
            let display_list = &command[1..];
            CommandResult::Show(display_list.join(" "))
        },
        "start" => CommandResult::Start,
        "move" => {
            match command.get(1).map(|n| n.parse::<usize>()) {
                Some(Ok(n)) => CommandResult::Move(n),
                _ => {
                    println!("usage: move <player number>");
                    CommandResult::CommandFailed
                }
            }
        },
        _ => unrecognised_command()
    }
}

fn handle_game_command(command: CommandResult, game: &mut Option<Game>) {
    match game {
        Some(ref mut g) => match command {
            CommandResult::Show(c) => println!("{}", g.show(&c)),
            CommandResult::Start => g.start(),
            CommandResult::Move(n) => {
                if let Err(e) = g.player_move(n) {
                    println!("move ignored: {}", e);
                }
            },
            _ => ()
        },
        None => match command {
            CommandResult::Show(_) | CommandResult::Start | CommandResult::Move(_) => {
                println!("no game loaded - use 'build <file>' first");
            },
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
        eprintln!("error: {}", lex_result.unwrap_err());
        return None;
    }

    let tokens = lex_result.expect("unable to unwrap tokens");

    let parse_result = parse::parse(&tokens);

    if parse_result.is_err() {
        eprintln!("error: {}", parse_result.unwrap_err());
        // maybe this should be an additional debug flag?
        //println!("{:?}", &tokens);
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