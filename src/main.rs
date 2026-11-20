use std::{fs, env, process, io::{stdin, stdout, IsTerminal, Write}};

use rustyline::{error::ReadlineError, history::DefaultHistory, CompletionType, Config, Editor};

mod lex;
mod parse;
mod token;
mod ast;
mod interpreter;
mod cards;
mod runtime;
mod move_input;
mod completion;

use interpreter::Game;

enum CommandResult {
    // boxed: a game is much larger than the other variants
    Game(Box<Game>),
    CommandFailed,
    Exit,
    Show(String),
    Start,
    Move(runtime::MoveInput),
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
        // build and start a game in one step
        "play"    => match args.get(2) {
            Some(path) => interactive(vec![format!("build {}", path), "start".to_string()]),
            None => {
                eprintln!("usage: cardlang play <file>");
                process::exit(1);
            }
        },
        _         => interactive(vec![])
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

// Runs the interactive prompt. `startup` lines run first, as if typed.
fn interactive(startup: Vec<String>) {
    println!("Cardlang interpreter");
    let mut game: Option<Game> = None;

    for line in startup {
        println!("> {}", line);
        if !run_line(&line, &mut game) {
            return;
        }
    }

    // line editing (tab completion, history) only makes sense in a terminal;
    // piped input (scripts, tests) is read plainly
    if stdin().is_terminal() {
        // like a shell: Tab completes as far as it can, Tab again lists the options
        let config = Config::builder().completion_type(CompletionType::List).build();
        match Editor::<completion::CliHelper, DefaultHistory>::with_config(config) {
            Ok(editor) => return interactive_terminal(editor, &mut game),
            Err(e) => eprintln!("line editing unavailable ({}), tab completion is off", e)
        }
    }
    interactive_plain(&mut game);
}

fn interactive_terminal(mut editor: Editor<completion::CliHelper, DefaultHistory>, game: &mut Option<Game>) {
    editor.set_helper(Some(completion::CliHelper::new()));
    loop {
        // keep completion in step with the game: hand, table, move names
        if let Some(helper) = editor.helper_mut() {
            helper.context = game.as_ref().map(|g| g.completion_context());
        }

        match editor.readline("> ") {
            Ok(line) => {
                if !line.trim().is_empty() {
                    let _ = editor.add_history_entry(line.as_str());
                }
                if !run_line(&line, game) {
                    break;
                }
            },
            // ctrl-c clears the line, like a shell
            Err(ReadlineError::Interrupted) => continue,
            // ctrl-d
            Err(ReadlineError::Eof) => {
                println!();
                break;
            },
            Err(e) => {
                eprintln!("error reading input: {}", e);
                break;
            }
        }
    }
}

fn interactive_plain(game: &mut Option<Game>) {
    loop {
        print!("> ");
        let _ = stdout().flush();

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            // end of input (e.g. a closed pipe)
            Ok(0) | Err(_) => {
                println!();
                break;
            },
            Ok(_) => ()
        }

        if !run_line(&input, game) {
            break;
        }
    }
}

// runs one line of input; returns false when the CLI should exit
fn run_line(input: &str, game: &mut Option<Game>) -> bool {
    let command = input.trim().split(' ').collect();
    let command_result = translate_command(command);

    // handle global commands
    match command_result {
        CommandResult::Game(ref g) => *game = Some((**g).clone()),
        CommandResult::Exit => return false,
        _ => ()
    }

    handle_game_command(command_result, game);
    true
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
        "help" => {
            print_help();
            CommandResult::Nothing
        },
        // anything else is a move, e.g. `take 7h / 3c 4s`
        _ => match move_input::parse_move(&command.join(" ")) {
            Ok(input) => CommandResult::Move(input),
            Err(e) => {
                println!("{}", e);
                CommandResult::CommandFailed
            }
        }
    }
}

fn print_help() {
    println!("commands:
  build <file>        load a game
  start               start (or restart) the game
  show <what>         game, name, players, current_player, deck, <stack>,
                      player <n> [stack]
  exit                quit
moves:
  [player] [name] [cards] [/ cards]
  e.g.  take 7h / 3c 4s     drop nine of diamonds     2 play
  cards: 7h, 10d, ks, as - or 'seven hearts', 'seven of hearts'
keys:
  Tab completes commands, file names, moves and the cards you can play
  Up/Down go through earlier lines");
}

fn handle_game_command(command: CommandResult, game: &mut Option<Game>) {
    match game {
        Some(ref mut g) => match command {
            // before `start` nothing has been dealt, so piles would look empty
            CommandResult::Show(c) if !g.is_started() && c != "game" && c != "name" => {
                println!("the game hasn't started - type 'start' to deal");
            },
            CommandResult::Show(c) => println!("{}", g.show(&c)),
            CommandResult::Start => {
                g.start();
                println!("{}", g.table());
            },
            CommandResult::Move(input) => {
                match g.make_move(input) {
                    Ok(()) => println!("{}", g.table()),
                    Err(e) => println!("move ignored: {}", e)
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

    let source = match fs::read_to_string(command[1]) {
        Ok(s) => s,
        Err(_) => {
            println!("unable to read '{}'", command[1]);
            return CommandResult::CommandFailed;
        }
    };

    match parse_game(source) {
        Some(g) => CommandResult::Game(Box::new(g)),
        None => CommandResult::CommandFailed
    }
}

fn parse_game(source: String) -> Option<Game> {
    let tokens = match lex::lexer(&source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {}", e);
            return None;
        }
    };

    let ast = match parse::parse(&tokens) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {}", e);
            return None;
        }
    };

    let game = Game::new(ast);
    println!("Game loaded");
    Some(game)
}
