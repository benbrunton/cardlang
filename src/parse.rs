use crate::token::{Token, SourceToken};
use crate::ast::*;

use std::iter::Peekable;

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

    // fills in a line number for errors raised where the line wasn't known
    pub fn or_line(self, line_number: u32) -> ParseError {
        if self.line_number == 0 {
            ParseError::new(self.error_type, line_number)
        } else {
            self
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self.error_type {
            ParseErrorType::ExpectedSymbol => "expected a name",
            ParseErrorType::UnexpectedEndOfStream => "unexpected end of statement - something is missing",
            ParseErrorType::UnexpectedToken => "unexpected word or symbol"
        };
        write!(f, "line {}: {}", self.line_number, message)
    }
}

pub fn parse(tokens: &Vec<SourceToken>) -> Result<Vec<Statement>, ParseError> {
    let mut ast = vec!();
    let mut tokens_iter = tokens.iter();

    loop {
        let current_token = tokens_iter.next();
        match current_token {
            Some(SourceToken{ token: Token::Players, line_number })
                if matches!(tokens_iter.clone().next(), Some(SourceToken{ token: Token::Transfer, .. })) => {
                // players can only receive cards, not be the source of a transfer
                return Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number));
            },
            Some(SourceToken{ token: declaration_token @ (Token::Name | Token::Players | Token::CurrentPlayer | Token::Stack), line_number }) => {
                let key = match get_key(declaration_token) {
                    Some(k) => k,
                    None => return Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number))
                };
                let next_token = match tokens_iter.next() {
                    Some(t) => t,
                    None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, *line_number))
                };
                let value = match (&key, &next_token.token) {
                    (GlobalKey::Name, Token::Symbol(_))
                        | (GlobalKey::Stack, Token::Symbol(_))
                        | (GlobalKey::Players, Token::Number(_))
                        | (GlobalKey::CurrentPlayer, Token::Number(_)) => get_value(&next_token.token),
                    _ => None
                };
                let value = match value {
                    Some(v) => v,
                    None => return Err(ParseError::new(ParseErrorType::UnexpectedToken, next_token.line_number))
                };

                // `stack <name> per player` gives every player their own stack
                let mut key = key;
                let mut lookahead = tokens_iter.clone();
                if key == GlobalKey::Stack {
                    if let (
                        Some(SourceToken{ token: Token::Symbol(per), .. }),
                        Some(SourceToken{ token: Token::Symbol(player), .. })
                    ) = (lookahead.next(), lookahead.next()) {
                        if per == "per" && player == "player" {
                            if value == Expression::Symbol("hand".to_string()) {
                                // every player already has a hand
                                return Err(ParseError::new(ParseErrorType::UnexpectedToken, next_token.line_number));
                            }
                            tokens_iter = lookahead;
                            key = GlobalKey::PlayerStack;
                        }
                    }
                }

                let declaration = Declaration{ key, value };
                let statement = Statement::Declaration(declaration);
                ast.push(statement);
            },
            Some(SourceToken{ token: Token::Deck, line_number }) => {
                let next_token_result = tokens_iter.next();
                match next_token_result {
                    Some(SourceToken{ token: Token::Symbol(deck_name), ..}) => {
                        let key = GlobalKey::Deck;
                        let value = Expression::Symbol(deck_name.to_owned());
                        let declaration = Declaration{ key, value };
                        let statement = Statement::Declaration(declaration);
                        ast.push(statement);
                    },
                    Some(SourceToken{ token: Token::Transfer, ..}) => {
                        let transfer_result = create_transfer("deck", &mut tokens_iter, *line_number);
                        if transfer_result.is_err() {
                            return Err(transfer_result.unwrap_err());
                        }
                        ast.push(transfer_result.unwrap())
                    },
                    None => {
                        return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, *line_number));
                    },
                    _ => {
                        return Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number)); 
                    }
                }
            },
            Some(SourceToken{ token: Token::Values, line_number }) => {
                let table = build_value_table(&mut tokens_iter, *line_number)?;
                ast.push(Statement::ValueTable(table));
            },
            Some(SourceToken{ token: Token::Define, line_number }) => {
                let next_token = match tokens_iter.next() {
                    Some(t) => t,
                    None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, *line_number))
                };
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
                        let transfer_result = create_transfer(name, &mut tokens_iter, *line_number);
                        if transfer_result.is_err() {
                            return Err(transfer_result.unwrap_err());
                        }
                        ast.push(transfer_result.unwrap())

                    },
                    _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number))
                }

 
            },
            Some(SourceToken{ token: Token::If, line_number }) => {
                let if_statement = build_if(&mut tokens_iter, *line_number)?;
                ast.push(Statement::IfStatement(if_statement));
            },
            Some(SourceToken{ token: Token::Check, line_number}) => {
                match tokens_iter.next() {
                    Some(SourceToken{ token: Token::OpenParens, ..}) => (),
                    _ => return Err(ParseError{
                        error_type: ParseErrorType::UnexpectedToken,
                        line_number: *line_number
                    })
                }

                let expression = build_expression(&mut tokens_iter)
                    .map_err(|e| e.or_line(*line_number))?;

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

                let expression = build_expression(&mut tokens_iter)
                    .map_err(|e| e.or_line(*line_number))?;

                let check_statement = ReturnStatement{ expression };
                let statement = Statement::ReturnStatement(check_statement);
                ast.push(statement);
            },
            None => { break; },
            Some(SourceToken{ token: Token::Newline, ..}) => (),
            Some(SourceToken{ token: Token::Test, line_number }) => {
                // spec test blocks are not supported yet - skip them
                skip_parens_group(&mut tokens_iter, *line_number)?;
            },
            Some(SourceToken{ line_number, ..}) => {
                return Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number));
            }
        }
    }

    Ok(ast)
}

fn create_function(name: &str, tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Statement, ParseError> {
    let arguments = build_call_arguments(tokens_iter)?;

    let function_call = FunctionCall { name: name.to_string(), arguments };
    Ok(Statement::FunctionCall(function_call))
}


fn create_transfer(from: &str, tokens_iter: &mut std::slice::Iter<SourceToken>, line_number: u32) -> Result<Statement, ParseError> {
    let transfer_target = match tokens_iter.next() {
        Some(t) => t,
        None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, line_number))
    };
    match transfer_target.token {
        Token::Deck | Token::Players | Token::Symbol(_) => (),
        Token::Newline => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, line_number)),
        _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, transfer_target.line_number))
    }
    let from = get_transfer_value(&Token::Symbol(from.to_string()));
    let to = get_transfer_value(&transfer_target.token);
    let modifier = None;
    let count = match tokens_iter.clone().next() {
        Some(SourceToken{ token: Token::Symbol(s), ..}) if s == "end" => {
            tokens_iter.next();
            Some(TransferCount::End)
        },
        Some(SourceToken{ token: Token::Number(n), line_number: l }) => {
            tokens_iter.next();
            if *n < 0.0 || n.fract() != 0.0 {
                return Err(ParseError::new(ParseErrorType::UnexpectedToken, *l));
            }
            Some(TransferCount::Number(*n as usize))
        },
        Some(SourceToken{ token: Token::Symbol(s), ..}) if s == "taking" => {
            tokens_iter.next();
            let cards = parse_or(tokens_iter).map_err(|e| e.or_line(line_number))?;
            Some(TransferCount::Taking(cards))
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

    // the block must start with '{', optionally on a following line
    loop {
        match tokens_iter.next() {
            Some(SourceToken{ token: Token::Newline, line_number: l }) => line_number = *l,
            Some(SourceToken{ token: Token::OpenBracket, line_number: l }) => {
                line_number = *l;
                break;
            },
            Some(SourceToken{ line_number: l, .. }) => {
                return Err(ParseError::new(ParseErrorType::UnexpectedToken, *l))
            },
            None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, line_number))
        }
    }

    // nested blocks stay in the body and are parsed recursively
    let mut depth = 0;
    loop {
        match tokens_iter.next() {
            Some(SourceToken{ token: Token::CloseBracket, .. }) if depth == 0 => break,
            Some(t) => {
                match t.token {
                    Token::OpenBracket => depth += 1,
                    Token::CloseBracket => depth -= 1,
                    _ => ()
                }
                line_number = t.line_number;
                body_tokens.push(t.clone());
            },
            None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, line_number))
        }
    }

    return parse(&body_tokens)
}

// Expression grammar, lowest precedence first:
//   expression := and_expr ( "or" and_expr )*
//   and_expr   := comparison ( "&" comparison )*
//   comparison := additive ( ( "is" ["not"] | "<" | ">" ) additive )?
//   additive   := primary ( ( "+" | "-" ) primary )*
//   primary    := literal | "-" number | name | name "(" arguments ")" | "(" expression ")"
// Parses the expression inside `check(...)`, `return(...)` or `if(...)`,
// consuming the closing parens.
fn build_expression(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Expression, ParseError> {
    let expression = parse_or(tokens_iter)?;
    match tokens_iter.next() {
        Some(SourceToken{ token: Token::CloseParens, ..}) => Ok(expression),
        Some(SourceToken{ line_number, .. }) => Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number)),
        None => Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, 0))
    }
}

// Parses a whole token group (e.g. one call argument) as a single expression.
fn build_complete_expression(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Expression, ParseError> {
    let expression = parse_or(tokens_iter)?;
    match tokens_iter.next() {
        None => Ok(expression),
        Some(SourceToken{ line_number, .. }) => Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number))
    }
}

fn peek_token(tokens_iter: &std::slice::Iter<SourceToken>) -> Option<Token> {
    tokens_iter.clone().next().map(|t| t.token.clone())
}

fn parse_or(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Expression, ParseError> {
    let mut left = parse_and(tokens_iter)?;
    while peek_token(tokens_iter) == Some(Token::Or) {
        tokens_iter.next();
        let right = parse_and(tokens_iter)?;
        left = Expression::Or(Box::new(Or{ left, right }));
    }
    Ok(left)
}

fn parse_and(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Expression, ParseError> {
    let mut left = parse_comparison(tokens_iter)?;
    while peek_token(tokens_iter) == Some(Token::Ampersand) {
        tokens_iter.next();
        let right = parse_comparison(tokens_iter)?;
        left = Expression::And(Box::new(And{ left, right }));
    }
    Ok(left)
}

fn parse_comparison(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Expression, ParseError> {
    let left = parse_additive(tokens_iter)?;
    let op = match peek_token(tokens_iter) {
        Some(Token::LessThan) => BinaryOp::LessThan,
        // inside an expression '>' means greater than, not a transfer
        Some(Token::Transfer) => BinaryOp::GreaterThan,
        Some(Token::Is) => {
            tokens_iter.next();
            let negative = peek_token(tokens_iter) == Some(Token::Not);
            if negative {
                tokens_iter.next();
            }
            let right = parse_additive(tokens_iter)?;
            return Ok(Expression::Comparison(Box::new(Comparison{ left, right, negative })));
        },
        _ => return Ok(left)
    };
    tokens_iter.next();
    let right = parse_additive(tokens_iter)?;
    Ok(Expression::Binary(Box::new(Binary{ op, left, right })))
}

fn parse_additive(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Expression, ParseError> {
    let mut left = parse_primary(tokens_iter)?;
    loop {
        let op = match peek_token(tokens_iter) {
            Some(Token::Plus) => BinaryOp::Add,
            Some(Token::Minus) => BinaryOp::Subtract,
            _ => return Ok(left)
        };
        tokens_iter.next();
        let right = parse_primary(tokens_iter)?;
        left = Expression::Binary(Box::new(Binary{ op, left, right }));
    }
}

fn parse_primary(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Expression, ParseError> {
    let source_token = match tokens_iter.next() {
        Some(t) => t,
        None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, 0))
    };

    let expression = match &source_token.token {
        Token::True => Expression::Bool(true),
        Token::False => Expression::Bool(false),
        Token::Number(n) => Expression::Number(*n),
        Token::Minus => match tokens_iter.next() {
            Some(SourceToken{ token: Token::Number(n), .. }) => Expression::Number(-*n),
            _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, source_token.line_number))
        },
        Token::CurrentPlayer => Expression::Symbol("current_player".to_string()),
        Token::Deck => Expression::Symbol("deck".to_string()),
        Token::Players => Expression::Symbol("players".to_string()),
        Token::Symbol(s) => {
            if peek_token(tokens_iter) == Some(Token::OpenParens) {
                tokens_iter.next();
                let arguments = build_call_arguments(tokens_iter)?;
                Expression::FunctionCall(FunctionCall{ name: s.to_string(), arguments })
            } else {
                Expression::Symbol(s.to_string())
            }
        },
        Token::OpenParens => {
            let inner = parse_or(tokens_iter)?;
            match tokens_iter.next() {
                Some(SourceToken{ token: Token::CloseParens, ..}) => inner,
                Some(SourceToken{ line_number, .. }) => {
                    return Err(ParseError::new(ParseErrorType::UnexpectedToken, *line_number))
                },
                None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, source_token.line_number))
            }
        },
        _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, source_token.line_number))
    };
    Ok(expression)
}
    

// Parses the rest of an if statement, after the `if` keyword.
fn build_if(tokens_iter: &mut std::slice::Iter<SourceToken>, line_number: u32) -> Result<IfStatement, ParseError> {
    match tokens_iter.next() {
        Some(SourceToken{ token: Token::OpenParens, ..}) => (),
        _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, line_number))
    }

    let expression = build_expression(tokens_iter).map_err(|e| e.or_line(line_number))?;
    let body = build_block(tokens_iter)?;
    let else_body = build_else(tokens_iter, line_number)?;

    Ok(IfStatement{ expression, body, else_body })
}

// Parses an optional `else { ... }` or `else if(...) { ... }` after an if block.
fn build_else(tokens_iter: &mut std::slice::Iter<SourceToken>, line_number: u32) -> Result<Vec<Statement>, ParseError> {
    // `else` may be on the line after the closing bracket
    let mut lookahead = tokens_iter.clone();
    loop {
        match lookahead.next() {
            Some(SourceToken{ token: Token::Newline, ..}) => (),
            Some(SourceToken{ token: Token::Else, ..}) => break,
            _ => return Ok(vec!())
        }
    }
    *tokens_iter = lookahead;

    if peek_token(tokens_iter) == Some(Token::If) {
        tokens_iter.next();
        let chained = build_if(tokens_iter, line_number)?;
        return Ok(vec!(Statement::IfStatement(chained)));
    }

    build_block(tokens_iter)
}

// `values { Jack 8, Queen 9, King 10 }` - commas and newlines between entries are optional
fn build_value_table(tokens_iter: &mut std::slice::Iter<SourceToken>, line_number: u32) -> Result<Vec<(String, f64)>, ParseError> {
    match tokens_iter.next() {
        Some(SourceToken{ token: Token::OpenBracket, .. }) => (),
        _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, line_number))
    }

    let mut table = vec!();
    loop {
        match tokens_iter.next() {
            Some(SourceToken{ token: Token::CloseBracket, .. }) => break,
            Some(SourceToken{ token: Token::Newline, .. }) | Some(SourceToken{ token: Token::Comma, .. }) => (),
            Some(SourceToken{ token: Token::Symbol(rank), line_number: l }) => {
                if !RANK_NAMES.contains(&rank.as_str()) {
                    return Err(ParseError::new(ParseErrorType::UnexpectedToken, *l));
                }
                let value = match tokens_iter.next() {
                    Some(SourceToken{ token: Token::Number(n), .. }) => *n,
                    Some(SourceToken{ token: Token::Minus, .. }) => match tokens_iter.next() {
                        Some(SourceToken{ token: Token::Number(n), .. }) => -*n,
                        _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, *l))
                    },
                    _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, *l))
                };
                table.push((rank.to_string(), value));
            },
            Some(SourceToken{ line_number: l, .. }) => return Err(ParseError::new(ParseErrorType::UnexpectedToken, *l)),
            None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, line_number))
        }
    }
    Ok(table)
}

fn skip_parens_group(tokens_iter: &mut std::slice::Iter<SourceToken>, line_number: u32) -> Result<(), ParseError> {
    match tokens_iter.next() {
        Some(SourceToken{ token: Token::OpenParens, ..}) => (),
        _ => return Err(ParseError::new(ParseErrorType::UnexpectedToken, line_number))
    }
    let mut depth = 1;
    while depth > 0 {
        match tokens_iter.next() {
            Some(SourceToken{ token: Token::OpenParens, ..}) => depth += 1,
            Some(SourceToken{ token: Token::CloseParens, ..}) => depth -= 1,
            Some(_) => (),
            None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, line_number))
        }
    }
    Ok(())
}

// Parses the comma separated arguments of a function call, consuming
// tokens up to and including the call's closing parens.
fn build_call_arguments(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Vec<Expression>, ParseError> {
    let mut groups: Vec<Vec<SourceToken>> = vec!();
    let mut current = vec!();
    let mut depth = 0;
    let mut line_number = 0;

    loop {
        let source_token = match tokens_iter.next() {
            Some(t) => t,
            None => return Err(ParseError::new(ParseErrorType::UnexpectedEndOfStream, line_number))
        };
        line_number = source_token.line_number;

        match source_token.token {
            Token::CloseParens if depth == 0 => break,
            Token::Comma if depth == 0 => {
                if current.is_empty() {
                    return Err(ParseError::new(ParseErrorType::UnexpectedToken, line_number));
                }
                groups.push(current);
                current = vec!();
            },
            Token::Newline => (),
            _ => {
                match source_token.token {
                    Token::OpenParens => depth += 1,
                    Token::CloseParens => depth -= 1,
                    _ => ()
                }
                current.push(source_token.clone());
            }
        }
    }

    if !current.is_empty() {
        groups.push(current);
    } else if !groups.is_empty() {
        // trailing comma
        return Err(ParseError::new(ParseErrorType::UnexpectedToken, line_number));
    }

    groups.iter().map(|group| {
        build_complete_expression(&mut group.iter()).map_err(|e| {
            let line = if e.line_number == 0 { group[0].line_number } else { e.line_number };
            ParseError::new(e.error_type, line)
        })
    }).collect()
}

fn build_args_list(tokens_iter: &mut std::slice::Iter<SourceToken>) -> Result<Vec<String>, ParseError> {
    let mut args_list = vec!();
    loop {
        match tokens_iter.next() {
            Some(SourceToken{ token: Token::Symbol(s), ..}) => args_list.push(s.to_string()),
            Some(SourceToken{ token: Token::Comma, ..}) => (),
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
    fn it_can_recognise_function_calls_with_multiple_arguments() {
        let tokens = get_source_tokens(vec!(
            Token::Symbol("deal".to_string()),
            Token::OpenParens,
            Token::Deck,
            Token::Comma,
            Token::Number(3.0),
            Token::CloseParens
        ));

        let function_call = FunctionCall{
            name: "deal".to_string(),
            arguments: vec!(
                Expression::Symbol("deck".to_string()),
                Expression::Number(3.0)
            )
        };
        let expected = Ok(vec!(Statement::FunctionCall(function_call)));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn it_can_parse_function_calls_with_no_arguments_in_expressions() {
        let tokens = get_source_tokens(vec!(
            Token::Return,
            Token::OpenParens,
            Token::Symbol("is_ready".to_string()),
            Token::OpenParens,
            Token::CloseParens,
            Token::CloseParens
        ));

        let expression = Expression::FunctionCall(FunctionCall{
            name: "is_ready".to_string(),
            arguments: vec!()
        });
        let expected = Ok(vec!(Statement::ReturnStatement(ReturnStatement{ expression })));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn it_can_parse_nested_function_calls_with_multiple_arguments_in_expressions() {
        let tokens = get_source_tokens(vec!(
            Token::Check,
            Token::OpenParens,
            Token::Symbol("same".to_string()),
            Token::OpenParens,
            Token::Symbol("count".to_string()),
            Token::OpenParens,
            Token::Symbol("player:hand".to_string()),
            Token::CloseParens,
            Token::Comma,
            Token::Number(2.0),
            Token::CloseParens,
            Token::Is,
            Token::True,
            Token::CloseParens
        ));

        let count = Expression::FunctionCall(FunctionCall{
            name: "count".to_string(),
            arguments: vec!(Expression::Symbol("player:hand".to_string()))
        });
        let same = Expression::FunctionCall(FunctionCall{
            name: "same".to_string(),
            arguments: vec!(count, Expression::Number(2.0))
        });
        let expression = Expression::Comparison(Box::new(Comparison{
            left: same,
            right: Expression::Bool(true),
            negative: false
        }));
        let expected = Ok(vec!(Statement::CheckStatement(CheckStatement{ expression })));

        assert_eq!(parse(&tokens), expected);
    }

    #[test]
    fn it_rejects_a_trailing_comma_in_function_calls() {
        let tokens = get_source_tokens(vec!(
            Token::Symbol("deal".to_string()),
            Token::OpenParens,
            Token::Deck,
            Token::Comma,
            Token::CloseParens
        ));

        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn it_accepts_commas_between_definition_arguments() {
        let tokens = get_source_tokens(vec!(
            Token::Define,
            Token::Symbol("take".to_owned()),
            Token::OpenParens,
            Token::Symbol("player".to_string()),
            Token::Comma,
            Token::Symbol("target".to_string()),
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        ));

        let definition = Definition{
            name: "take".to_owned(),
            arguments: vec!("player".to_string(), "target".to_string()),
            body: vec!()
        };
        let expected = Ok(vec!(Statement::Definition(definition)));

        assert_eq!(parse(&tokens), expected);
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
        let if_statement = IfStatement{ expression, body, else_body: vec!() };
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
        let if_statement = IfStatement{ expression, body, else_body: vec!() };
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
            right: Expression::Number(1.0),
            negative: false
        };
        let expression = Expression::Comparison(Box::new(comparison));
        let body = vec!();
        let if_statement = IfStatement{ expression, body, else_body: vec!() };
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
        let if_statement = IfStatement{ expression, body, else_body: vec!() };
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
            right: Expression::Number(0.0),
            negative: false
        };
        let expression = Expression::Comparison(Box::new(comparison));
        let body = vec!();
        let if_statement = IfStatement{ expression, body, else_body: vec!() };
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
                IfStatement{ else_body: vec!(),
                    expression: Expression::Comparison(Box::new(Comparison{
                        left: Expression::FunctionCall(FunctionCall{
                            name: "count".to_string(),
                            arguments: vec!(
                                Expression::Symbol("player:hand".to_string())
                            )
                        }),
                        right: Expression::Number(0.0),
                        negative:false
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
        def player_move(player){
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
                IfStatement{ else_body: vec!(),
                    expression: Expression::Comparison(Box::new(Comparison{
                        left: Expression::FunctionCall(FunctionCall{
                            name: "count".to_string(),
                            arguments: vec!(
                                Expression::Symbol("player:hand".to_string())
                            )
                        }),
                        right: Expression::Number(0.0),
                        negative: false
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
            right: Expression::Symbol("player:id".to_string()),
            negative: false
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

    #[test]
    fn it_can_handle_negative_comparisons() {
        let tokens = get_source_tokens(vec!(
            Token::If,
            Token::OpenParens,
            Token::Symbol("player:id".to_string()),
            Token::Is,
            Token::Not,
            Token::Number(1.0),
            Token::CloseParens,
            Token::OpenBracket,
            Token::CloseBracket
        ));

        let comparison = Comparison {
            left: Expression::Symbol("player:id".to_string()),
            right: Expression::Number(1.0),
            negative: true
        };
        let expression = Expression::Comparison(Box::new(comparison));
        let body = vec!();
        let if_statement = IfStatement{ expression, body, else_body: vec!() };
        let statement = Statement::IfStatement(if_statement);
        let expected = vec!(statement);
        let result = parse(&tokens);

        assert_eq!(Ok(expected), result);
    }

    fn parse_source(source: &str) -> Result<Vec<Statement>, ParseError> {
        let tokens = crate::lex::lexer(source).expect("lex error");
        parse(&tokens)
    }

    #[test]
    fn incomplete_or_invalid_source_is_a_parse_error_not_a_panic() {
        let cases = [
            "values { Joker 1 }",
            "values { King }",
            "values King 10",
            "def setup(){ check(1 + ) }",
            "def setup(){ check(1 < 2 < 3) }",
            "stack hand per player",
            "def setup() }",
            "def setup() x { }",
            "}",
            ")",
            "5",
            "name x 5",
            "def setup(){ deck > middle taking }",
            "def setup(){ if true){ end() } }",
            "name",
            "players",
            "stack",
            "current_player",
            "deck",
            "players two",
            "name 5",
            "stack 3",
            "current_player first",
            "players > deck end",
            "def",
            "def setup(){ check(1 is) }",
            "def setup(){ check(1 &) }",
            "def setup(){ check(1 is not) }",
            "def setup(){ x > }",
            "def setup(){ deck > }",
            "def setup(){ deck > 5 }",
        ];

        for case in cases.iter() {
            assert!(parse_source(case).is_err(), "expected a parse error for {:?}", case);
        }
    }

    #[test]
    fn an_unfinished_transfer_reports_its_line() {
        let result = parse_source("name x\ndef setup(){\n    deck >\n}");
        assert_eq!(result.unwrap_err().line_number, 3);
    }

    #[test]
    fn the_parser_never_panics_on_random_tokens() {
        let alphabet = vec!(
            Token::Name, Token::Symbol("a".to_string()), Token::Symbol("p:hand".to_string()),
            Token::Number(1.0), Token::Stack, Token::Deck, Token::Players, Token::CurrentPlayer,
            Token::Define, Token::OpenParens, Token::CloseParens, Token::Comma,
            Token::OpenBracket, Token::CloseBracket, Token::Transfer, Token::Check, Token::Is,
            Token::If, Token::Newline, Token::True, Token::False, Token::Ampersand,
            Token::Return, Token::Test, Token::Not, Token::Or, Token::Else,
            Token::Plus, Token::Minus, Token::LessThan, Token::Values
        );

        // simple deterministic pseudo random generator, so failures are reproducible
        let mut seed: u64 = 0x2545F4914F6CDD1D;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };

        for _ in 0..20000 {
            let length = (next() % 12) as usize;
            let tokens: Vec<Token> = (0..length)
                .map(|_| alphabet[(next() % alphabet.len() as u64) as usize].clone())
                .collect();
            let source_tokens = get_source_tokens(tokens.clone());
            let result = std::panic::catch_unwind(|| parse(&source_tokens));
            assert!(result.is_ok(), "parser panicked on {:?}", tokens);
        }
    }

    #[test]
    fn spec_test_blocks_are_skipped() {
        let result = parse_source("name x\n.test(\n    anything (nested) here\n)\nplayers 2");
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn comparisons_bind_tighter_than_and() {
        let result = parse_source("return(a is 1 & b is not 2)").unwrap();

        let expected = Expression::And(Box::new(And{
            left: Expression::Comparison(Box::new(Comparison{
                left: Expression::Symbol("a".to_string()),
                right: Expression::Number(1.0),
                negative: false
            })),
            right: Expression::Comparison(Box::new(Comparison{
                left: Expression::Symbol("b".to_string()),
                right: Expression::Number(2.0),
                negative: true
            }))
        }));

        assert_eq!(result, vec!(Statement::ReturnStatement(ReturnStatement{ expression: expected })));
    }

    #[test]
    fn invalid_expressions_and_else_are_parse_errors() {
        let cases = [
            "def setup(){ check(1 is 2 is 3) }",
            "def setup(){ check(or true) }",
            "def setup(){ check(true &) }",
            "def setup(){ check((true) }",
            "def setup(){ if(true){ } else }",
            "def setup(){ else { } }",
        ];

        for case in cases.iter() {
            assert!(parse_source(case).is_err(), "expected a parse error for {:?}", case);
        }
    }

    #[test]
    fn it_parses_per_player_stacks() {
        let result = parse_source("stack middle\nstack collection per player").unwrap();

        assert_eq!(result, vec!(
            Statement::Declaration(Declaration{ key: GlobalKey::Stack, value: Expression::Symbol("middle".to_string()) }),
            Statement::Declaration(Declaration{ key: GlobalKey::PlayerStack, value: Expression::Symbol("collection".to_string()) })
        ));
    }

    #[test]
    fn arithmetic_binds_tighter_than_comparison() {
        let result = parse_source("return(a + 1 > b - 2)").unwrap();

        let expected = Expression::Binary(Box::new(Binary{
            op: BinaryOp::GreaterThan,
            left: Expression::Binary(Box::new(Binary{
                op: BinaryOp::Add,
                left: Expression::Symbol("a".to_string()),
                right: Expression::Number(1.0)
            })),
            right: Expression::Binary(Box::new(Binary{
                op: BinaryOp::Subtract,
                left: Expression::Symbol("b".to_string()),
                right: Expression::Number(2.0)
            }))
        }));

        assert_eq!(result, vec!(Statement::ReturnStatement(ReturnStatement{ expression: expected })));
    }

    #[test]
    fn it_parses_a_value_table() {
        let result = parse_source("values {\n    Ace 11, King 10\n    Two -1\n}").unwrap();
        let expected = vec!(Statement::ValueTable(vec!(
            ("Ace".to_string(), 11.0),
            ("King".to_string(), 10.0),
            ("Two".to_string(), -1.0)
        )));
        assert_eq!(result, expected);
    }
}
