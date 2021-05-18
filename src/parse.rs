use crate::token::Token;
use crate::ast::*;

// use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError{
    ExpectedSymbol,
    UnexpectedEndOfStream,
    UnexpectedToken
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ParseError> {
    let mut ast = vec!();
    let mut tokens_iter = tokens.iter();

    loop {
        let current_token = tokens_iter.next();
        match current_token {
            Some(Token::Name) | Some(Token::Players) | Some(Token::CurrentPlayer)
                | Some(Token::Stack) => {
                let unwrapped_token = current_token.expect("unable to unwrap token");
                let key = get_key(unwrapped_token).expect("unable to find key");
                let next_token = tokens_iter.next().expect("unable to find next token");
                let value = get_value(next_token).expect("unable to find expression");
                let declaration = Declaration{ key, value };
                let statement = Statement::Declaration(declaration);
                ast.push(statement);
            },
            Some(Token::Deck) => {
                let deck_token = current_token.expect("unable to unwrap token");
                let next_token_result = tokens_iter.next();
                match next_token_result {
                    Some(Token::Symbol(_)) => {
                        let key = get_key(deck_token).expect("unable to find key");
                        let next_token = next_token_result.expect("unable to find next token");
                        let value = get_value(next_token).expect("unable to find expression");
                        let declaration = Declaration{ key, value };
                        let statement = Statement::Declaration(declaration);
                        ast.push(statement);
                    },
                    Some(Token::Transfer) => {
                        let transfer_result = create_transfer("deck", &mut tokens_iter);
                        if transfer_result.is_err() {
                            return Err(transfer_result.unwrap_err());
                        }
                        ast.push(transfer_result.unwrap())
                    },
                    _ => { return Err(ParseError::UnexpectedToken); }
                }
            },
            Some(Token::Define) => {
                let next_token = tokens_iter.next().expect("unable to find next token");
                let name = match next_token {
                    Token::Symbol(s) => s.to_owned(),
                    _ => {
                        return Err(ParseError::ExpectedSymbol)
                    }
                };

                // parens
                tokens_iter.next();
                tokens_iter.next();

                // bracket
                tokens_iter.next();

                let body = match build_block(&mut tokens_iter) {
                    Ok(b) => b,
                    Err(e) => return Err(e)
                };

                let definition = Definition{ name, body };
                let statement = Statement::Definition(definition);
                ast.push(statement);
            },
            Some(Token::Symbol(name)) => {
                match tokens_iter.next() {
                    Some(Token::OpenParens) => {
                        let func_result = create_function(name, &mut tokens_iter);
                        if func_result.is_err() {
                            return Err(func_result.unwrap_err());
                        }
                        ast.push(func_result.unwrap());
                    },
                    Some(Token::Transfer) => {
                        let transfer_result = create_transfer(name, &mut tokens_iter);
                        if transfer_result.is_err() {
                            return Err(transfer_result.unwrap_err());
                        }
                        ast.push(transfer_result.unwrap())

                    },
                    _ => return Err(ParseError::UnexpectedToken)
                }

 
            },
            Some(Token::If) => {
                tokens_iter.next(); // assuming open parens?

                let expression = match build_expression(&mut tokens_iter) {
                    Ok(ex) => ex,
                    Err(e) => return Err(e)
                };

                //tokens_iter.next();  // close parens?

                let body = match build_block(&mut tokens_iter) {
                    Ok(b) => b,
                    Err(e) => return Err(e)
                };

                let if_statement = IfStatement{ expression, body };
                let statement = Statement::IfStatement(if_statement);
                ast.push(statement);
            },
            None => { break; },
            _ => (),
        }
    }

    Ok(ast)
}

fn create_function(name: &str, tokens_iter: &mut std::slice::Iter<Token>) -> Result<Statement, ParseError> {
    let mut arguments = vec!();

    match tokens_iter.next() {
        Some(Token::Deck) => {
            arguments.push(Expression::Symbol("deck".to_string()));
        },
        Some(Token::Symbol(s)) => {
            arguments.push(Expression::Symbol(s.to_string()));
        },
        _ => ()
    };

    //close parens
    //tokens_iter.next();

    let function_call = FunctionCall { name: name.to_string(), arguments };
    Ok(Statement::FunctionCall(function_call))
}


fn create_transfer(from: &str, tokens_iter: &mut std::slice::Iter<Token>) -> Result<Statement, ParseError> {
    let transfer_target = tokens_iter.next().expect("unable to find next token");
    let from = get_transfer_value(&Token::Symbol(from.to_string()));
    let to = get_transfer_value(transfer_target);
    let modifier = None;
    let count = match tokens_iter.next() {
        Some(Token::Symbol(s)) => {
            if s == "end" {
                Some(TransferCount::End)
            } else {
                None
            }
        },
        _ => None
    };

    let transfer = Transfer{ from, to, modifier, count };
    let statement = Statement::Transfer(transfer);
    Ok(statement)
}


fn get_key(token: &Token) -> Option<GlobalKey> {
    match token {
        Token::Name => Some(GlobalKey::Name),
        Token::Players => Some(GlobalKey::Players),
        Token::Deck => Some(GlobalKey::Deck),
        Token::CurrentPlayer => Some(GlobalKey::CurrentPlayer),
        Token::Stack => Some(GlobalKey::Stack),
        _ => None
    }
}

fn get_value(token: &Token) -> Option<Expression> {
    match token {
        Token::Symbol(a) => Some(Expression::Symbol(a.to_owned())),
        Token::Number(a) => Some(Expression::Number(*a)),
        _ => None
    }
}

fn get_transfer_value(token: &Token) -> String {
    match token {
        Token::Deck => "deck".to_owned(),
        Token::Players => "players".to_owned(),
        Token::Symbol(s) => s.to_owned(),
        _ => "".to_owned() // todo - handle errors
    }
}

fn build_block(tokens_iter: &mut std::slice::Iter<Token>) -> Result<Vec<Statement>, ParseError> {
    let mut body_tokens = vec!();

    loop {
        match tokens_iter.next() {
            Some(Token::CloseBracket) => {
                break;
            },
            Some(t) => {
                body_tokens.push(t.clone());
            },
            None => return Err(ParseError::UnexpectedEndOfStream)
        }
    }

    return parse(body_tokens)
}

fn build_expression(tokens_iter: &mut std::slice::Iter<Token>) -> Result<Expression, ParseError> {
    let left = match tokens_iter.next() {
        Some(Token::True) => Expression::Bool(true),
        Some(Token::False) => Expression::Bool(false),
        Some(Token::Symbol(s)) => Expression::Symbol(s.to_string()),
        Some(Token::Number(n)) => Expression::Number(*n),
        None => return Err(ParseError::UnexpectedEndOfStream),
        _ => return Err(ParseError::UnexpectedToken)
    };
    combine_expression(tokens_iter, left)
}

fn combine_expression(tokens_iter: &mut std::slice::Iter<Token>, left: Expression) -> Result<Expression, ParseError> {
    match tokens_iter.next() {
        None | Some(Token::CloseParens) => Ok(left),
        Some(Token::Is) => {
            let right = build_expression(tokens_iter).expect("bad right expression");
            let comparison = Comparison {
                left,
                right
            };
            Ok(Expression::Comparison(Box::new(comparison)))
        },
        Some(Token::OpenParens) => {
            match left {
                Expression::Symbol(s) => {
                    let arguments = vec!(build_expression(tokens_iter).expect("bad args!"));
                    let function = FunctionCall{
                        name: s.to_string(),
                        arguments
                    };
                    combine_expression(tokens_iter, Expression::FunctionCall(function))
                },
                _ => Err(ParseError::UnexpectedToken)
            }
        },
        _ => Err(ParseError::UnexpectedToken)
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn it_can_handle_a_single_declaration() {
        let tokens = vec!(
            Token::Name,
            Token::Symbol("turns".to_string())
        );
        let mut expected = vec!();
        let key = GlobalKey::Name;
        let value = Expression::Symbol("turns".to_string());
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let result = parse(tokens);

        assert_eq!(Ok(expected), result)
    }

    #[test]
    fn it_can_handle_numerical_declaration(){ 
        let tokens = vec!(
            Token::Players,
            Token::Number(2.0)
        );
        let mut expected = vec!();
        let key = GlobalKey::Players;
        let value = Expression::Number(2.0);
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let result = parse(tokens);

        assert_eq!(Ok(expected), result)
    }

    #[test]
    fn it_can_handle_newlines(){ 
        let tokens = vec!(
            Token::Name,
            Token::Symbol("turns".to_string()),
            Token::Newline,
            Token::Players,
            Token::Number(2.0)
        );
        let mut expected = vec!();
        let key = GlobalKey::Name;
        let value = Expression::Symbol("turns".to_string());
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let key = GlobalKey::Players;
        let value = Expression::Number(2.0);
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let result = parse(tokens);

        assert_eq!(Ok(expected), result)
    }

    #[test]
    fn it_can_setup_a_simple_game() {
        let tokens = vec!(
            Token::Name,
            Token::Symbol("turns".to_string()),
            Token::Newline,
            Token::Players,
            Token::Number(2.0),
            Token::Deck,
            Token::Symbol("StandardDeck".to_string()),
            Token::CurrentPlayer,
            Token::Number(1.0),
            Token::Stack,
            Token::Symbol("middle".to_owned())
        );

        let mut expected = vec!();
        let key = GlobalKey::Name;
        let value = Expression::Symbol("turns".to_string());
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let key = GlobalKey::Players;
        let value = Expression::Number(2.0);
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let key = GlobalKey::Deck;
        let value = Expression::Symbol("StandardDeck".to_string());
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let key = GlobalKey::CurrentPlayer;
        let value = Expression::Number(1.0);
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let key = GlobalKey::Stack;
        let value = Expression::Symbol("middle".to_string());
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);
        let result = parse(tokens);

        assert_eq!(Ok(expected), result)
    }

    #[test]
    fn it_recognises_a_function_definition() {
        let tokens = vec!(
            Token::Define,
            Token::Symbol("setup".to_owned()),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        );

        let name = "setup".to_owned();
        let body = vec!();
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);
        let expected = vec!(statement);
        let result = parse(tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_returns_a_parse_error_when_function_not_defined_correctly() {
        let tokens = vec!(
            Token::Define,
            Token::Number(1.0),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        );

        let expected = Err(ParseError::ExpectedSymbol);
        let result = parse(tokens);

        assert_eq!(expected, result);
    }

    // deck > players alt end
    #[test]
    fn it_recognises_stack_transfers() {
        let tokens = vec!(
            Token::Deck,
            Token::Transfer,
            Token::Players
        );

        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let statement = Statement::Transfer(transfer);
        let expected = Ok(vec!(statement));
        let result = parse(tokens);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_can_handle_function_body() {
        let tokens = vec!(
            Token::Define,
            Token::Symbol("setup".to_owned()),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket,
            Token::Deck,
            Token::Transfer,
            Token::Players,
            Token::Newline,
            Token::CloseBracket
        );

        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec!(transfer_statement);
        let definition = Definition{ name, body };
        let statement = Statement::Definition(definition);
        let expected = vec!(statement);
        let result = parse(tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_returns_error_for_incomplete_function_body() {
        let tokens = vec!(
            Token::Define,
            Token::Symbol("setup".to_owned()),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket
        );

        let expected = Err(ParseError::UnexpectedEndOfStream);
        let result = parse(tokens);

        assert_eq!(expected, result);
    }

    
    #[test]
    fn it_returns_error_for_invalid_function_body() {
        let tokens = vec!(
            Token::Define,
            Token::Symbol("setup".to_owned()),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket,
            Token::Define,
            Token::Newline,
            Token::CloseBracket
        );

        let expected = Err(ParseError::ExpectedSymbol);
        let result = parse(tokens);

        assert_eq!(expected, result);
    }

    #[test]
    fn deck_must_be_followed_by_a_symbol_or_transfer() {
        let tokens = vec!(
            Token::Deck,
            Token::Players
        );

        let expected = Err(ParseError::UnexpectedToken);
        let result = parse(tokens);

        assert_eq!(expected, result);
    }

    #[test]
    fn it_can_recognise_function_calls() {
        let tokens = vec!(
            Token::Symbol("shuffle".to_string()), Token::OpenParens,
            Token::Deck, Token::CloseParens
        );

        let function_call = FunctionCall{
            name: "shuffle".to_string(),
            arguments: vec!(Expression::Symbol("deck".to_string()))
        };
        let statement = Statement::FunctionCall(function_call);
        let expected = Ok(vec!(statement));

        let result = parse(tokens);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_recognises_player_hand_to_deck_transfer() {
        let tokens = vec!(
            Token::Symbol("player:hand".to_string()),
            Token::Transfer,
            Token::Deck
        );

        let from = "player:hand".to_owned();
        let to = "deck".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let statement = Statement::Transfer(transfer);
        let expected = Ok(vec!(statement));
        
        let result = parse(tokens);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_can_pass_a_count_to_transfer() {
        let tokens = vec!(
            Token::Symbol("player:hand".to_string()),
            Token::Transfer,
            Token::Deck,
            Token::Symbol("end".to_string())
        );

        let from = "player:hand".to_owned();
        let to = "deck".to_owned();
        let modifier = None;
        let count = Some(TransferCount::End);
        let transfer = Transfer{ from, to, modifier, count };
        let statement = Statement::Transfer(transfer);
        let expected = Ok(vec!(statement));
        
        let result = parse(tokens);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_can_recognise_function_calls_with_no_arguments() {
        let tokens = vec!(
            Token::Symbol("end".to_string()),
            Token::OpenParens,
            Token::CloseParens
        );

        let function_call = FunctionCall{
            name: "end".to_string(),
            arguments: vec!()
        };

        let statement = Statement::FunctionCall(function_call);
        let expected = Ok(vec!(statement));

        let result = parse(tokens);

        assert_eq!(result, expected);
    }

    #[test]
    fn does_it_recognise_win_player_id() {
        let tokens = vec!(
            Token::Symbol("winner".to_string()),
            Token::OpenParens,
            Token::Symbol("player:id".to_string()),
            Token::CloseParens
        );

        let function_call = FunctionCall{
            name: "winner".to_string(),
            arguments: vec!(Expression::Symbol("player:id".to_string()))
        };

        let statement = Statement::FunctionCall(function_call);
        let expected = Ok(vec!(statement));

        let result = parse(tokens);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_can_handle_if_statements() {
        let tokens = vec!(
            Token::If,
            Token::OpenParens,
            Token::True,
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        );
        let expression = Expression::Bool(true);
        let body = vec!();
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_can_handle_false_if_statements() {
        let tokens = vec!(
            Token::If,
            Token::OpenParens,
            Token::False,
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        );
        let expression = Expression::Bool(false);
        let body = vec!();
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_can_handle_comparisons_in_if_statement() {
        let tokens = vec!(
            Token::If,
            Token::OpenParens,
            Token::Symbol("player:id".to_string()),
            Token::Is,
            Token::Number(1.0),
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        );

        let comparison = Comparison {
            left: Expression::Symbol("player:id".to_string()),
            right: Expression::Number(1.0)
        };
        let expression = Expression::Comparison(Box::new(comparison));
        let body = vec!();
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_assigns_statements_to_an_if_statement() {
        let tokens = vec!(
            Token::If,
            Token::OpenParens,
            Token::True,
            Token::CloseParens,
            Token::OpenBracket,
            Token::Symbol("shuffle".to_string()),
            Token::OpenParens,
            Token::Deck,
            Token::CloseParens,
            Token::CloseBracket
        );
        let expression = Expression::Bool(true);
        let function_call = FunctionCall{
            name: "shuffle".to_string(),
            arguments: vec!(Expression::Symbol("deck".to_string()))
        };
        let body = vec!(Statement::FunctionCall(function_call));
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(tokens);

        assert_eq!(Ok(expected), result);
    }

    // if(count(player:hand) is 0)
    #[test]
    fn it_can_handle_func_calls_in_comparisons() {
        let tokens = vec!(
            Token::If,
            Token::OpenParens,
            Token::Symbol("count".to_string()),
            Token::OpenParens,
            Token::Symbol("player:hand".to_string()),
            Token::CloseParens,
            Token::Is,
            Token::Number(0.0),
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        );

        let function_call = FunctionCall{
            name: "count".to_string(),
            arguments: vec!(
                Expression::Symbol("player:hand".to_string())
            )
        };

        let comparison = Comparison {
            left: Expression::FunctionCall(function_call),
            right: Expression::Number(0.0)
        };
        let expression = Expression::Comparison(Box::new(comparison));
        let body = vec!();
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(tokens);

        assert_eq!(Ok(expected), result);
    }
}
        