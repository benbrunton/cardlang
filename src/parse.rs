use crate::token::{Token, SourceToken};
use crate::ast::*;

// use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseErrorType{
    ExpectedSymbol,
    UnexpectedEndOfStream,
    UnexpectedToken
}


#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub error_type: ParseErrorType,
    pub line_number: u32
}

impl ParseError {
    pub fn new(error_type: ParseErrorType, line_number: u32) -> ParseError {
        ParseError {
            error_type,
            line_number
        }
    }
}

pub fn parse(tokens: &Vec<SourceToken>) -> Result<Vec<Statement>, ParseError> {
    let mut ast = vec!();
    let mut tokens_iter = tokens.iter();

    loop {
        let current_token = tokens_iter.next();
        match current_token {
            Some(SourceToken{ token: Token::Name, .. }) 
                | Some(SourceToken{ token: Token::Players, ..}) 
                | Some(SourceToken{ token: Token::CurrentPlayer, ..})
                | Some(SourceToken{ token: Token::Stack, ..}) => {
                let unwrapped_token = current_token.expect("unable to unwrap token");
                let key = get_key(&unwrapped_token.token).expect("unable to find key");
                let next_token = tokens_iter.next().expect("unable to find next token");
                let value = get_value(&next_token.token).expect("unable to find expression");
                let declaration = Declaration{ key, value };
                let statement = Statement::Declaration(declaration);
                ast.push(statement);
            },
            Some(SourceToken{ token: Token::Deck, line_number }) => {
                let deck_token = current_token.expect("unable to unwrap token");
                let next_token_result = tokens_iter.next();
                match next_token_result {
                    Some(SourceToken{ token: Token::Symbol(_), ..}) => {
                        let key = get_key(&deck_token.token).expect("unable to find key");
                        let next_token = next_token_result.expect("unable to find next token");
                        let value = get_value(&next_token.token).expect("unable to find expression");
                        let declaration = Declaration{ key, value };
                        let statement = Statement::Declaration(declaration);
                        ast.push(statement);
                    },
                    Some(SourceToken{ token: Token::Transfer, ..}) => {
                        let transfer_result = create_transfer("deck", &mut tokens_iter);
                        if transfer_result.is_err() {
                            return Err(transfer_result.unwrap_err());
                        }
                        ast.push(transfer_result.unwrap())
                    },
                    _ => {
                        return Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number)); 
                    }
                }
            },
            Some(SourceToken{ token: Token::Define, ..}) => {
                let next_token = tokens_iter.next().expect("unable to find next token");
                let name = match &next_token.token {
                    Token::Symbol(s) => s.to_owned(),
                    _ => {
                        return Err(
                            ParseError::new(
                                ParseErrorType::ExpectedSymbol, next_token.line_number
                            )
                        )
                    }
                };

                // parens
                tokens_iter.next();

                let arguments = match build_args_list(&mut tokens_iter){
                    Ok(args) => args,
                    Err(e) => return Err(e)
                };

                let body = match build_block(&mut tokens_iter) {
                    Ok(b) => b,
                    Err(e) => return Err(e)
                };

                let definition = Definition{ arguments, name, body };
                let statement = Statement::Definition(definition);
                ast.push(statement);
            },
            Some(SourceToken{token: Token::Symbol(name), line_number }) => {
                match tokens_iter.next() {
                    Some(SourceToken{ token: Token::OpenParens, ..}) => {
                        let func_result = create_function(name, &mut tokens_iter);
                        if func_result.is_err() {
                            return Err(func_result.unwrap_err());
                        }
                        ast.push(func_result.unwrap());
                    },
                    Some(SourceToken{ token: Token::Transfer, ..}) => {
                        let transfer_result = create_transfer(name, &mut tokens_iter);
                        if transfer_result.is_err() {
                            return Err(transfer_result.unwrap_err());
                        }
                        ast.push(transfer_result.unwrap())

                    },
                    _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number))
                }

 
            },
            Some(SourceToken{ token: Token::If, ..}) => {
                tokens_iter.next(); // assuming open parens?

                let expression = match build_expression(&mut tokens_iter) {
                    Ok(ex) => ex,
                    Err(e) => return Err(e)
                };

                let body = match build_block(&mut tokens_iter) {
                    Ok(b) => b,
                    Err(e) => return Err(e)
                };

                let if_statement = IfStatement{ expression, body };
                let statement = Statement::IfStatement(if_statement);
                ast.push(statement);
            },
            Some(SourceToken{ token: Token::Check, line_number}) => {
                match tokens_iter.next() {
                    Some(SourceToken{ token: Token::OpenParens, ..}) => (),
                    _ => return Err(ParseError{
                        error_type: ParseErrorType::UnexpectedToken,
                        line_number: *line_number
                    })
                }

                let expression = match build_expression(&mut tokens_iter) {
                    Ok(ex) => ex,
                    Err(e) => return Err(e)
                };

                let check_statement = CheckStatement{ expression };
                let statement = Statement::CheckStatement(check_statement);
                ast.push(statement);
            },
            Some(SourceToken{ token: Token::Return, line_number}) => {
                match tokens_iter.next() {
                    Some(SourceToken{ token: Token::OpenParens, ..}) => (),
                    _ => return Err(ParseError{
                        error_type: ParseErrorType::UnexpectedToken,
                        line_number: *line_number
                    })
                }

                let expression = match build_expression(&mut tokens_iter) {
                    Ok(ex) => ex,
                    Err(e) => return Err(e)
                };

                let check_statement = ReturnStatement{ expression };
                let statement = Statement::ReturnStatement(check_statement);
                ast.push(statement);
            },
            None => { break; },
            _ => (),
        }
    }

    Ok(ast)
}

fn create_function(name: &str, tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Statement, ParseError> {
    let mut arguments = vec!();

    match tokens_iter.next() {
        Some(SourceToken{ token: Token::Deck, ..}) => {
            arguments.push(Expression::Symbol("deck".to_string()));
        },
        Some(SourceToken{ token: Token::Symbol(s), ..}) => {
            arguments.push(Expression::Symbol(s.to_string()));
        },
        _ => ()
    };

    //close parens
    //tokens_iter.next();

    let function_call = FunctionCall { name: name.to_string(), arguments };
    Ok(Statement::FunctionCall(function_call))
}


fn create_transfer(from: &str, tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Statement, ParseError> {
    let transfer_target = tokens_iter.next().expect("unable to find next token");
    let from = get_transfer_value(&Token::Symbol(from.to_string()));
    let to = get_transfer_value(&transfer_target.token);
    let modifier = None;
    let count = match tokens_iter.next() {
        Some(SourceToken{ token: Token::Symbol(s), ..}) => {
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

fn build_block(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Vec<Statement>, ParseError> {
    let mut body_tokens = vec!();
    let mut line_number = 0;
    let mut open_bracket_count = 0;

    loop {
        match tokens_iter.next() {
            Some(SourceToken{ token: Token::CloseBracket, line_number }) => {
                if open_bracket_count > 1 {
                    open_bracket_count -= 1;
                    body_tokens.push(
                        SourceToken{ token: Token::CloseBracket, line_number: *line_number }
                    );
                } else {
                    break;
                }
            },
            Some(t) => {
                if t.token == Token::OpenBracket {
                    open_bracket_count += 1;
                }
                line_number = t.line_number;
                body_tokens.push(t.clone());
            },
            None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, line_number))
        }
    }

    return parse(&body_tokens)
}

fn build_expression(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Expression, ParseError> {
    let left = match tokens_iter.next() {
        Some(SourceToken{ token: Token::True, ..}) => Expression::Bool(true),
        Some(SourceToken{ token: Token::False, ..}) => Expression::Bool(false),
        Some(SourceToken{ token: Token::Symbol(s), ..}) => Expression::Symbol(s.to_string()),
        Some(SourceToken{ token: Token::Number(n), ..}) => Expression::Number(*n),
        Some(SourceToken{ token: Token::CurrentPlayer, ..}) => Expression::Symbol("current_player".to_string()),
        None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, 0)),
        _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, 0))
    };
    combine_expression(tokens_iter, left)
}

fn combine_expression(tokens_iter: &mut std::slice::Iter<SourceToken>, left: Expression) -> Result<Expression, ParseError> {
    match tokens_iter.next() {
        None | Some(SourceToken{ token: Token::CloseParens, ..}) => Ok(left),
        Some(SourceToken{ token: Token::Is, ..}) => {
            let right = build_expression(tokens_iter).expect("bad right expression");
            let comparison = Comparison {
                left,
                right
            };
            Ok(Expression::Comparison(Box::new(comparison)))
        },
        Some(SourceToken{ token: Token::Ampersand, ..}) => {
            let right = build_expression(tokens_iter).expect("bad right expression");
            let and = And {
                left,
                right
            };
            Ok(Expression::And(Box::new(and)))
        },
        Some(SourceToken{ token: Token::OpenParens, ..}) => {
            match left {
                Expression::Symbol(s) => {
                    let arguments = vec!(build_expression(tokens_iter).expect("bad args!"));
                    let function = FunctionCall{
                        name: s.to_string(),
                        arguments
                    };
                    combine_expression(tokens_iter, Expression::FunctionCall(function))
                },
                _ => Err(ParseError::new(ParseErrorType::UnexpectedToken, 0))
            }
        },
        _ => Err(ParseError::new(ParseErrorType::UnexpectedToken, 0))
    }
}
    

fn build_args_list(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Vec<String>, ParseError> {
    let mut args_list = vec!();
    loop {
        match tokens_iter.next() {
            Some(SourceToken{ token: Token::Symbol(s), ..}) => args_list.push(s.to_string()),
            Some(SourceToken{ token: Token::CloseParens, ..}) => break,
            Some(SourceToken{ line_number, .. }) => {
                return Err(ParseError::new(ParseErrorType::ExpectedSymbol, *line_number))
            },
            None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, 0))
        }
    }

    Ok(args_list)
}

#[cfg(test)]
mod test{
    use super::*;

    fn get_source_tokens(tokens: Vec<Token>) -> Vec<SourceToken> {
        tokens.iter().map(|t| SourceToken{ token: t.to_owned(), line_number: 0 }).collect()
    }

    #[test]
    fn it_can_handle_a_single_declaration() {
        let tokens = get_source_tokens(vec!(
            Token::Name,
            Token::Symbol("turns".to_string())
        ));
        let mut expected = vec!();
        let key = GlobalKey::Name;
        let value = Expression::Symbol("turns".to_string());
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let result = parse(&tokens);

        assert_eq!(Ok(expected), result)
    }

    #[test]
    fn it_can_handle_numerical_declaration(){ 
        let tokens = get_source_tokens(vec!(
            Token::Players,
            Token::Number(2.0)
        ));
        let mut expected = vec!();
        let key = GlobalKey::Players;
        let value = Expression::Number(2.0);
        let declaration = Declaration{ key, value };

        let statement = Statement::Declaration(declaration);
        expected.push(statement);

        let result = parse(&tokens);

        assert_eq!(Ok(expected), result)
    }

    #[test]
    fn it_can_handle_newlines(){ 
        let tokens = get_source_tokens(vec!(
            Token::Name,
            Token::Symbol("turns".to_string()),
            Token::Newline,
            Token::Players,
            Token::Number(2.0)
        ));
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

        let result = parse(&tokens);

        assert_eq!(Ok(expected), result)
    }

    #[test]
    fn it_can_setup_a_simple_game() {
        let tokens = get_source_tokens(vec!(
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
        ));

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
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result)
    }

    #[test]
    fn it_recognises_a_function_definition() {
        let tokens = get_source_tokens(vec!(
            Token::Define,
            Token::Symbol("setup".to_owned()),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        ));

        let name = "setup".to_owned();
        let body = vec!();
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);
        let expected = vec!(statement);
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_returns_a_parse_error_when_function_not_defined_correctly() {
        let tokens = get_source_tokens(vec!(
            Token::Define,
            Token::Number(1.0),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        ));

        let expected = ParseErrorType::ExpectedSymbol;
        let result = parse(&tokens);

        assert_eq!(result.unwrap_err().error_type, expected);
    }

    // deck > players alt end
    #[test]
    fn it_recognises_stack_transfers() {
        let tokens = get_source_tokens(vec!(
            Token::Deck,
            Token::Transfer,
            Token::Players
        ));

        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let statement = Statement::Transfer(transfer);
        let expected = Ok(vec!(statement));
        let result = parse(&tokens);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_can_handle_function_body() {
        let tokens = get_source_tokens(vec!(
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
        ));

        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let transfer_statement = Statement::Transfer(transfer);

        let name = "setup".to_owned();
        let body = vec!(transfer_statement);
        let definition = Definition{ arguments: vec!(), name, body };
        let statement = Statement::Definition(definition);
        let expected = vec!(statement);
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_returns_error_for_incomplete_function_body() {
        let tokens = get_source_tokens(vec!(
            Token::Define,
            Token::Symbol("setup".to_owned()),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket
        ));

        let expected = ParseErrorType::UnexpectedEndOfStream;
        let result = parse(&tokens);

        assert_eq!(result.unwrap_err().error_type, expected);
    }

    
    #[test]
    fn it_returns_error_for_invalid_function_body() {
        let tokens = get_source_tokens(vec!(
            Token::Define,
            Token::Symbol("setup".to_owned()),
            Token::OpenParens,
            Token::CloseParens,
            Token::OpenBracket,
            Token::Define,
            Token::Newline,
            Token::CloseBracket
        ));

        let expected = ParseErrorType::ExpectedSymbol;
        let result = parse(&tokens);

        assert_eq!(result.unwrap_err().error_type, expected);
    }

    #[test]
    fn deck_must_be_followed_by_a_symbol_or_transfer() {
        let tokens = get_source_tokens(vec!(
            Token::Deck,
            Token::Players
        ));

        let expected = ParseErrorType::UnexpectedToken;
        let result = parse(&tokens);

        assert_eq!(result.unwrap_err().error_type, expected);
    }

    #[test]
    fn it_can_recognise_function_calls() {
        let tokens = get_source_tokens(vec!(
            Token::Symbol("shuffle".to_string()), Token::OpenParens,
            Token::Deck, Token::CloseParens
        ));

        let function_call = FunctionCall{
            name: "shuffle".to_string(),
            arguments: vec!(Expression::Symbol("deck".to_string()))
        };
        let statement = Statement::FunctionCall(function_call);
        let expected = Ok(vec!(statement));

        let result = parse(&tokens);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_recognises_player_hand_to_deck_transfer() {
        let tokens = get_source_tokens(vec!(
            Token::Symbol("player:hand".to_string()),
            Token::Transfer,
            Token::Deck
        ));

        let from = "player:hand".to_owned();
        let to = "deck".to_owned();
        let modifier = None;
        let count = None;
        let transfer = Transfer{ from, to, modifier, count };
        let statement = Statement::Transfer(transfer);
        let expected = Ok(vec!(statement));
        
        let result = parse(&tokens);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_can_pass_a_count_to_transfer() {
        let tokens = get_source_tokens(vec!(
            Token::Symbol("player:hand".to_string()),
            Token::Transfer,
            Token::Deck,
            Token::Symbol("end".to_string())
        ));

        let from = "player:hand".to_owned();
        let to = "deck".to_owned();
        let modifier = None;
        let count = Some(TransferCount::End);
        let transfer = Transfer{ from, to, modifier, count };
        let statement = Statement::Transfer(transfer);
        let expected = Ok(vec!(statement));
        
        let result = parse(&tokens);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_can_recognise_function_calls_with_no_arguments() {
        let tokens = get_source_tokens(vec!(
            Token::Symbol("end".to_string()),
            Token::OpenParens,
            Token::CloseParens
        ));

        let function_call = FunctionCall{
            name: "end".to_string(),
            arguments: vec!()
        };

        let statement = Statement::FunctionCall(function_call);
        let expected = Ok(vec!(statement));

        let result = parse(&tokens);

        assert_eq!(result, expected);
    }

    #[test]
    fn does_it_recognise_win_player_id() {
        let tokens = get_source_tokens(vec!(
            Token::Symbol("winner".to_string()),
            Token::OpenParens,
            Token::Symbol("player:id".to_string()),
            Token::CloseParens
        ));

        let function_call = FunctionCall{
            name: "winner".to_string(),
            arguments: vec!(Expression::Symbol("player:id".to_string()))
        };

        let statement = Statement::FunctionCall(function_call);
        let expected = Ok(vec!(statement));

        let result = parse(&tokens);

        assert_eq!(result, expected);
    }

    #[test]
    fn it_can_handle_if_statements() {
        let tokens = get_source_tokens(vec!(
            Token::If,
            Token::OpenParens,
            Token::True,
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        ));
        let expression = Expression::Bool(true);
        let body = vec!();
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_can_handle_false_if_statements() {
        let tokens = get_source_tokens(vec!(
            Token::If,
            Token::OpenParens,
            Token::False,
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        ));
        let expression = Expression::Bool(false);
        let body = vec!();
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_can_handle_comparisons_in_if_statement() {
        let tokens = get_source_tokens(vec!(
            Token::If,
            Token::OpenParens,
            Token::Symbol("player:id".to_string()),
            Token::Is,
            Token::Number(1.0),
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        ));

        let comparison = Comparison {
            left: Expression::Symbol("player:id".to_string()),
            right: Expression::Number(1.0)
        };
        let expression = Expression::Comparison(Box::new(comparison));
        let body = vec!();
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_assigns_statements_to_an_if_statement() {
        let tokens = get_source_tokens(vec!(
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
        ));
        let expression = Expression::Bool(true);
        let function_call = FunctionCall{
            name: "shuffle".to_string(),
            arguments: vec!(Expression::Symbol("deck".to_string()))
        };
        let body = vec!(Statement::FunctionCall(function_call));
        let if_statement = IfStatement{ expression, body };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }

    // if(count(player:hand) is 0)
    #[test]
    fn it_can_handle_func_calls_in_comparisons() {
        let tokens = get_source_tokens(vec!(
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
        ));

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
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn it_returns_a_line_number_on_errors() {
        let tokens = vec!(
            SourceToken{ token: Token::Define, line_number: 1 },
            SourceToken{ token: Token::Number(1.0), line_number: 1 },
            SourceToken{ token: Token::OpenParens, line_number: 1 },
            SourceToken{ token: Token::CloseParens, line_number: 1 },
            SourceToken{ token: Token::OpenBracket, line_number: 1 },
            SourceToken{ token: Token::CloseBracket, line_number: 1 },
        );

        let expected = ParseError::new(ParseErrorType::ExpectedSymbol, 1);
        let result = parse(&tokens);

        assert_eq!(result.unwrap_err(), expected);
    }

    #[test]
    fn it_returns_a_line_number_on_more_errors() {
        let tokens = vec!(
            SourceToken{ token: Token::Deck, line_number: 2 },
            SourceToken{ token: Token::CloseBracket, line_number: 2 },
        );

        let expected = ParseError::new(ParseErrorType::UnexpectedToken, 2);
        let result = parse(&tokens);

        assert_eq!(result.unwrap_err(), expected);
    }

    #[test]
    fn it_returns_a_line_number_on_unexpected_token_after_symbol() {
        let tokens = vec!(
            SourceToken{ token: Token::Symbol("foo".to_string()), line_number: 3 },
            SourceToken{ token: Token::Symbol("bar".to_string()), line_number: 3 },
        );

        let expected = ParseError::new(ParseErrorType::UnexpectedToken, 3);
        let result = parse(&tokens);

        assert_eq!(result.unwrap_err(), expected);
    }

    #[test]
    fn it_returns_a_line_number_on_unexpected_end_of_stream() {
        let tokens = vec!(
            SourceToken{ token: Token::If, line_number: 4 },
            SourceToken{ token: Token::OpenParens, line_number: 4 },
            SourceToken{ token: Token::Symbol("player:id".to_string()), line_number: 4 },
            SourceToken{ token: Token::Is, line_number: 4 },
            SourceToken{ token: Token::Number(1.0), line_number: 4 },
            SourceToken{ token: Token::CloseParens, line_number: 4 },
            SourceToken{ token: Token::Newline, line_number: 4 },
            SourceToken{ token: Token::OpenBracket, line_number: 5 }
        );

        let expected = ParseError::new(ParseErrorType::UnexpectedEndOfStream, 5);
        let result = parse(&tokens);

        assert_eq!(result.unwrap_err(), expected);
    }

    #[test]
    fn it_can_parse_a_multiline_if_block() {
        /*
        if(count(player:hand) is 0){
            winner(player:id)
            end()
        }
        */

        let tokens = vec!(
            SourceToken{ token: Token::If, line_number: 0 },
            SourceToken{ token: Token::OpenParens, line_number: 0 },
            SourceToken{ token: Token::Symbol("count".to_string()), line_number: 0 },
            SourceToken{ token: Token::OpenParens, line_number: 0 },
            SourceToken{ token: Token::Symbol("player:hand".to_string()), line_number: 0 },
            SourceToken{ token: Token::CloseParens, line_number: 0 },
            SourceToken{ token: Token::Is, line_number: 0 },
            SourceToken{ token: Token::Number(0.0), line_number: 0 },
            SourceToken{ token: Token::CloseParens, line_number: 0 },
            SourceToken{ token: Token::OpenBracket, line_number: 0 },
            SourceToken{ token: Token::Newline, line_number: 0 },
            SourceToken{ token: Token::Symbol("winner".to_string()), line_number: 1 },
            SourceToken{ token: Token::OpenParens, line_number: 1 },
            SourceToken{ token: Token::Symbol("player:id".to_string()), line_number: 1 },
            SourceToken{ token: Token::CloseParens, line_number: 1 },
            SourceToken{ token: Token::Newline, line_number: 1 },
            SourceToken{ token: Token::Symbol("end".to_string()), line_number: 2 },
            SourceToken{ token: Token::OpenParens, line_number: 2 },
            SourceToken{ token: Token::CloseParens, line_number: 2 },
            SourceToken{ token: Token::Newline, line_number: 2 },
            SourceToken{ token: Token::CloseBracket, line_number: 3 },
            SourceToken{ token: Token::Newline, line_number: 3 },
        );

        let expected = vec!(
            Statement::IfStatement(
                IfStatement{
                    expression: Expression::Comparison(Box::new(Comparison{
                        left: Expression::FunctionCall(FunctionCall{
                            name: "count".to_string(),
                            arguments: vec!(
                                Expression::Symbol("player:hand".to_string())
                            )
                        }),
                        right: Expression::Number(0.0)
                    })),
                    body: vec!(
                        Statement::FunctionCall(FunctionCall{
                            name: "winner".to_string(),
                            arguments: vec!(Expression::Symbol("player:id".to_string()))
                        }),
                        Statement::FunctionCall(FunctionCall{
                            name: "end".to_string(),
                            arguments: vec!()
                        })
                    )
                }
            )
        );
        let result = parse(&tokens);

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn it_can_parse_a_multiline_if_block_inside_a_func() {
        /*
        define player_move(player){
            if(count(player:hand) is 0){
                winner(player:id)
                end()
            }
        }
        */

        let tokens = vec!(
            SourceToken{ token: Token::Define, line_number: 1 },
            SourceToken{ token: Token::Symbol("player_move".to_string()), line_number: 1 },
            SourceToken{ token: Token::OpenParens, line_number: 1 },
            SourceToken{ token: Token::Symbol("player".to_string()), line_number: 1 },
            SourceToken{ token: Token::CloseParens, line_number: 1 },
            SourceToken{ token: Token::OpenBracket, line_number: 1 },
            SourceToken{ token: Token::Newline, line_number: 1 },
            SourceToken{ token: Token::If, line_number: 2 },
            SourceToken{ token: Token::OpenParens, line_number: 2 },
            SourceToken{ token: Token::Symbol("count".to_string()), line_number: 2 },
            SourceToken{ token: Token::OpenParens, line_number: 2 },
            SourceToken{ token: Token::Symbol("player:hand".to_string()), line_number: 2 },
            SourceToken{ token: Token::CloseParens, line_number: 2 },
            SourceToken{ token: Token::Is, line_number: 2 },
            SourceToken{ token: Token::Number(0.0), line_number: 2 },
            SourceToken{ token: Token::CloseParens, line_number: 2 },
            SourceToken{ token: Token::OpenBracket, line_number: 2 },
            SourceToken{ token: Token::Newline, line_number: 2 },
            SourceToken{ token: Token::Symbol("winner".to_string()), line_number: 3 },
            SourceToken{ token: Token::OpenParens, line_number: 3 },
            SourceToken{ token: Token::Symbol("player:id".to_string()), line_number: 3 },
            SourceToken{ token: Token::CloseParens, line_number: 3 },
            SourceToken{ token: Token::Newline, line_number: 3 },
            SourceToken{ token: Token::Symbol("end".to_string()), line_number: 4 },
            SourceToken{ token: Token::OpenParens, line_number: 4 },
            SourceToken{ token: Token::CloseParens, line_number: 4 },
            SourceToken{ token: Token::Newline, line_number: 4 },
            SourceToken{ token: Token::CloseBracket, line_number: 5 },
            SourceToken{ token: Token::Newline, line_number: 5 },
            SourceToken{ token: Token::CloseBracket, line_number: 6 },
            SourceToken{ token: Token::Newline, line_number: 6 },
        );

        let body = vec!(
            Statement::IfStatement(
                IfStatement{
                    expression: Expression::Comparison(Box::new(Comparison{
                        left: Expression::FunctionCall(FunctionCall{
                            name: "count".to_string(),
                            arguments: vec!(
                                Expression::Symbol("player:hand".to_string())
                            )
                        }),
                        right: Expression::Number(0.0)
                    })),
                    body: vec!(
                        Statement::FunctionCall(FunctionCall{
                            name: "winner".to_string(),
                            arguments: vec!(Expression::Symbol("player:id".to_string()))
                        }),
                        Statement::FunctionCall(FunctionCall{
                            name: "end".to_string(),
                            arguments: vec!()
                        })
                    )
                }
            )
        );

        let expected = vec!(
            Statement::Definition(Definition{
                name: "player_move".to_string(),
                body,
                arguments: vec!("player".to_string()),
            })
        );
        let result = parse(&tokens);

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn it_can_parse_a_check_statement() {
        let tokens = vec!(
            SourceToken{ token: Token::Check, line_number: 1 },
            SourceToken{ token: Token::OpenParens, line_number: 1 },
            SourceToken{ token: Token::True, line_number: 1 },
            SourceToken{ token: Token::CloseParens, line_number: 1 },
        );

        let expected = vec!(
            Statement::CheckStatement(CheckStatement{
                expression: Expression::Bool(true)
            })
        );

        let result = parse(&tokens);

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn it_insists_on_an_open_parens_for_check_condition() {
        let tokens = vec!(
            SourceToken{ token: Token::Check, line_number: 1 },
            SourceToken{ token: Token::True, line_number: 1 }
        );

        let expected = ParseError{
            error_type: ParseErrorType::UnexpectedToken,
            line_number: 1
        };

        let result = parse(&tokens);

        assert_eq!(result, Err(expected));
    }

    #[test]
    fn it_can_parse_a_check_statement_with_current_player() {
        let tokens = vec!(
            SourceToken{ token: Token::Check, line_number: 1 },
            SourceToken{ token: Token::OpenParens, line_number: 1 },
            SourceToken{ token: Token::CurrentPlayer, line_number: 1 },
            SourceToken{ token: Token::Is, line_number: 1 },
            SourceToken{ token: Token::Symbol("player:id".to_string()), line_number: 1 },
            SourceToken{ token: Token::CloseParens, line_number: 1 },
        );

        let expression = Expression::Comparison(Box::new(Comparison{
            left: Expression::Symbol("current_player".to_string()),
            right: Expression::Symbol("player:id".to_string())
        }));

        let expected = vec!(
            Statement::CheckStatement(CheckStatement{ expression })
        );

        let result = parse(&tokens);

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn it_can_parse_a_return_statement() {
        let tokens = vec!(
            SourceToken{ token: Token::Return, line_number: 1 },
            SourceToken{ token: Token::OpenParens, line_number: 1 },
            SourceToken{ token: Token::True, line_number: 1 },
            SourceToken{ token: Token::CloseParens, line_number: 1 },
        );

        let expected = vec!(
            Statement::ReturnStatement(ReturnStatement{
                expression: Expression::Bool(true)
            })
        );

        let result = parse(&tokens);

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn it_can_parse_an_and_statement() {
        let tokens = vec!(
            SourceToken{ token: Token::Return, line_number: 1 },
            SourceToken{ token: Token::OpenParens, line_number: 1 },
            SourceToken{ token: Token::True, line_number: 1 },
            SourceToken{ token: Token::Ampersand, line_number: 1 },
            SourceToken{ token: Token::True, line_number: 1 },
            SourceToken{ token: Token::CloseParens, line_number: 1 },
        );

        let expected = vec!(
            Statement::ReturnStatement(ReturnStatement{
                expression: Expression::And(Box::new(And{
                    left: Expression::Bool(true),
                    right: Expression::Bool(true)
                }))
            })
        );

        let result = parse(&tokens);

        assert_eq!(result, Ok(expected));

    }

    #[test]
    fn it_parses_the_argument_of_a_function() {
        let tokens = get_source_tokens(vec!(
            Token::Define,
            Token::Symbol("not_royal".to_owned()),
            Token::OpenParens,
            Token::Symbol("card".to_string()),
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        ));

        let name = "not_royal".to_owned();
        let body = vec!();
        let definition = Definition{ arguments: vec!("card".to_string()), name, body };
        let statement = Statement::Definition(definition);
        let expected = vec!(statement);
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }
}
        