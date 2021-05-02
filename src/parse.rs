use crate::token::Token;
use crate::ast::*;

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
                        let transfer_target = tokens_iter.next().expect("unable to find next token");
                        let from = get_transfer_value(deck_token);
                        let to = get_transfer_value(transfer_target);
                        let modifier = Some(TransferModifier::Alternate);
                        let count = Some(TransferCount::End);
                        let transfer = Transfer{ from, to, modifier, count };
                        let statement = Statement::Transfer(transfer);
                        ast.push(statement);
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

                let body = match parse(body_tokens) {
                    Ok(v) => v,
                    Err(e) => { return Err(e); }
                };

                let definition = Definition{ name, body };
                let statement = Statement::Definition(definition);
                ast.push(statement);
            },
            None => { break; },
            _ => (),
        }
    }

    Ok(ast)
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
        _ => "".to_owned() // todo - handle errors
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
            Token::Players,
            Token::Symbol("alt".to_owned()),
            Token::Symbol("end".to_owned())
        );

        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = Some(TransferModifier::Alternate);
        let count = Some(TransferCount::End);
        let transfer = Transfer{ from, to, modifier, count };
        let statement = Statement::Transfer(transfer);
        let expected = Ok(vec!(statement));
        let result = parse(tokens);

        assert_eq!(result, expected);
    }

    /*
    define setup(){
        deck > players alt end
    }
    */
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
            Token::Symbol("alt".to_owned()),
            Token::Symbol("end".to_owned()),
            Token::Newline,
            Token::CloseBracket
        );

        let from = "deck".to_owned();
        let to = "players".to_owned();
        let modifier = Some(TransferModifier::Alternate);
        let count = Some(TransferCount::End);
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
}