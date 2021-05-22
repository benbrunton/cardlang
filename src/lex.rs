use crate::token::{Token, SourceToken};

enum TokenResult {
    Token(Token),
    PartialToken(String),
    Empty,
    Error
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexErrorType{
    EmptySpecification,
    ParseError
}

#[derive(Debug, PartialEq, Clone)]
pub struct LexError{
    pub error_type: LexErrorType,
    pub line_number: u32
}

impl LexError{
    pub fn new(error_type: LexErrorType, line_number: u32) -> LexError {
        LexError{ error_type, line_number }
    }
}

pub fn lexer(source: &str) -> Result<Vec<SourceToken>, LexError> {
    let mut line_number = 1;
    let mut tokens = vec!();
    let mut chars = source.chars().peekable();

    let mut partial_token: Option<String> = None;

    loop {
        let current_char_result = chars.next();
        if current_char_result.is_none() {
            break;
        }

        let current_char = current_char_result.expect("expected a char");
        let next_char = chars.peek();
        let result = handle_char(&partial_token, current_char, next_char);

        match result {
            TokenResult::Token(t) => {
                if t == Token::Newline {
                    line_number += 1;
                }

                partial_token = None;
                let source_token = SourceToken{
                    token: t,
                    line_number
                };
                tokens.push(source_token);
            },
            TokenResult::PartialToken(s) => {
                partial_token = Some(s);
            },
            TokenResult::Empty => {
                partial_token = None;
            },
            TokenResult::Error => {
                let lex_error = LexError::new(LexErrorType::ParseError, line_number);
                return Err(lex_error);
            }
        }
    }

    if tokens.len() == 0 {
        let lex_error = LexError::new(LexErrorType::EmptySpecification, line_number);
        Err(lex_error)
    } else {
        Ok(tokens)
    }
}


fn handle_char(partial_token: &Option<String>, current_char: char, next_char: Option<&char>) -> TokenResult {
    match partial_token {
        None => {
            let single_char = handle_single_chars(current_char);
            if single_char.is_some() {
                return single_char.expect("should have a single char");
            }
            
            handle_partial(current_char.to_string(), next_char)
        },
        Some(x) => {
            let new_partial = format!("{}{}", x, current_char);
            handle_partial(new_partial, next_char)
        }
    }
}

fn handle_single_chars(current_char: char) -> Option<TokenResult> {
    match current_char {
        '(' => Some(TokenResult::Token(Token::OpenParens)),
        ')' => Some(TokenResult::Token(Token::CloseParens)),
        ' ' => Some(TokenResult::Empty),
        ',' => Some(TokenResult::Token(Token::Comma)),
        '{' => Some(TokenResult::Token(Token::OpenBracket)),
        '}' => Some(TokenResult::Token(Token::CloseBracket)),
        '>' => Some(TokenResult::Token(Token::Transfer)),
        '\n' => Some(TokenResult::Token(Token::Newline)),
        '.' => Some(TokenResult::PartialToken(current_char.to_string())),
        _ => None
    }
}

fn handle_partial(current_partial: String, next_char: Option<&char>) -> TokenResult {
    let keyword_result = handle_keyword(&current_partial, next_char);

    if keyword_result.is_some() {
        return keyword_result.expect("should be a keyword");
    }

    if is_word_finished(next_char) {
        return resolve_partial(current_partial);
    }

    TokenResult::PartialToken(current_partial)
}

fn handle_keyword(partial_token: &str, next_char: Option<&char>) -> Option<TokenResult> {
    if !is_word_finished(next_char) {
        return None
    }

    match partial_token {
        "name" => Some(TokenResult::Token(Token::Name)),
        "stack" => Some(TokenResult::Token(Token::Stack)),
        "deck" => Some(TokenResult::Token(Token::Deck)),
        "players" => Some(TokenResult::Token(Token::Players)),
        "current_player" => Some(TokenResult::Token(Token::CurrentPlayer)),
        "define" => Some(TokenResult::Token(Token::Define)),
        "check" => Some(TokenResult::Token(Token::Check)),
        "is" => Some(TokenResult::Token(Token::Is)),
        "if" => Some(TokenResult::Token(Token::If)),
        "true" => Some(TokenResult::Token(Token::True)),
        "false" => Some(TokenResult::Token(Token::False)),
        _ => None
    }
}

fn resolve_partial(partial_token: String) -> TokenResult {
    let mut chars = partial_token.chars();
    let first = chars.next().expect("unable to find first char in partial token");
    match first {
        'A'..='z' => TokenResult::Token(Token::Symbol(partial_token)),
        '.' => {
            match chars.next() {
                // comments
                Some('(') => {
                    let mut open_count = 0;
                    loop {
                        match chars.next() {
                            Some('(') => open_count += 1,
                            Some(')') => {
                                if open_count == 0 {
                                    return TokenResult::Empty;
                                }

                                open_count -= 1;
                            },
                            None => break,
                            _ => ()
                        }
                    }
                    TokenResult::PartialToken(partial_token)
                },
                _ => TokenResult::Error
            }
        },
        _ => {
            let parse_result = partial_token.parse::<f64>();
            match parse_result {
                Ok(float) => TokenResult::Token(Token::Number(float)),
                _ => TokenResult::Error
            }
            
        }
    }
    
}

fn is_word_finished(next_char: Option<&char>) -> bool {
    match next_char {
        Some('A'..='z') | Some('0'..='9') | Some(':') => {
            false
        },
        _ => true
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn empty_string_is_not_a_valid_prog() {
        let src = "";
        let result = lexer(&src).unwrap_err();

        assert_eq!(result.error_type, LexErrorType::EmptySpecification);
    }

    #[test]
    fn it_recognises_the_name_token() {
        let src = "name";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Name);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn it_recognises_the_stack_token() {
        let src = "stack";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Stack);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn it_recognises_the_deck_keyword() {
        let src = "deck";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Deck);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn it_recognises_keywords(){
        let src = "deck players stack current_player define";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Deck);
        assert_eq!(result[1].token, Token::Players);
        assert_eq!(result[2].token, Token::Stack);
        assert_eq!(result[3].token, Token::CurrentPlayer);
        assert_eq!(result[4].token, Token::Define);
    }

    #[test]
    fn it_recognises_an_arbitrary_symbol() {
        let src = "scopa";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Symbol("scopa".to_owned()));
    }

    #[test]
    fn it_recognises_camel_case() {
        let src = "StandardDeck";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Symbol("StandardDeck".to_owned()));
    }

    #[test]
    fn it_handles_full_lines() {
        let src = "deck StandardDeck";
        let result = lexer(&src).unwrap();
        let expected = vec!(
            Token::Deck,
            Token::Symbol("StandardDeck".to_owned())
        );
        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
    }

    #[test]
    fn it_handles_open_parens() {
        let src = "(";
        let result = lexer(&src).unwrap();
        let expected = Token::OpenParens;
        assert_eq!(result[0].token, expected);
    }

    #[test]
    fn it_handles_close_parens() {
        let src = ")";
        let result = lexer(&src).unwrap();
        let expected = Token::CloseParens;
        assert_eq!(result[0].token, expected);
    }

    #[test]
    fn it_handles_comma() {
        let src = ",";
        let result = lexer(&src).unwrap();
        let expected = Token::Comma;
        assert_eq!(result[0].token, expected);
    }

    #[test]
    fn it_handles_brackets() {
        let src = "{}";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::OpenBracket, Token::CloseBracket);
        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
    }

    #[test]
    fn it_handles_transfer() {
        let src = ">";
        let result = lexer(&src).unwrap();
        let expected = Token::Transfer;
        assert_eq!(result[0].token, expected);
    }

    #[test]
    fn it_handles_check_and_is() {
        let src = "check cards is fun";
        let result = lexer(&src).unwrap();
        let expected = vec!(
            Token::Check, Token::Symbol("cards".to_owned()),
            Token::Is, Token::Symbol("fun".to_owned())
        );
        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
    }

    #[test]
    fn it_handles_if(){
        let src ="if";
        let result = lexer(&src).unwrap();
        let expected = Token::If;
        assert_eq!(result[0].token, expected);
    }

    #[test]
    fn it_handles_numbers(){
        let src ="1";
        let result = lexer(&src).unwrap();
        let expected = Token::Number(1.0);
        assert_eq!(result[0].token, expected);
    }

    #[test]
    fn symbols_cant_start_with_a_num() {
        let src = "1foo";
        let result = lexer(&src).unwrap_err();

        assert_eq!(result.error_type, LexErrorType::ParseError);
    }

    #[test]
    fn symbols_can_contain_a_num() {
        let src = "foo1";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Symbol("foo1".to_owned()));
    }

    #[test]
    fn newlines_are_tokenised() {
        let src = "\n";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Newline);
    }

    #[test]
    fn it_ignores_comments() {
        let src = "name .( this is a comment ) test1";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::Name, Token::Symbol("test1".to_owned()));
        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
    }

    #[test]
    fn comments_can_be_multiline() {
        let src = "name .( 
this is a comment ) test2";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::Name, Token::Symbol("test2".to_owned()));
        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
    }

    #[test]
    fn comments_can_contain_parens() {
        let src = "name .(()) test2";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::Name, Token::Symbol("test2".to_owned()));
        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
    }

    #[test]
    fn symbols_can_contain_underscores() {
        let src = "hello_world";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Symbol("hello_world".to_owned()));
    }

    #[test]
    fn it_recognises_function_calls() {
        let src = "shuffle(deck)";
        let result = lexer(&src).unwrap();
        let expected = vec!(
            Token::Symbol("shuffle".to_string()), Token::OpenParens,
            Token::Deck, Token::CloseParens
        );
        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
        assert_eq!(result[2].token, expected[2]);
        assert_eq!(result[3].token, expected[3]);
    }

    #[test]
    fn a_symbol_can_contain_an_attribute() {
        let src = "player:hand";
        let result = lexer(&src).unwrap();
        let expected = Token::Symbol("player:hand".to_owned());
        assert_eq!(result[0].token, expected)
    }

    #[test]
    fn it_can_recognise_true_and_false(){
        let src = "true false";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::True, Token::False);
        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
    }

    #[test]
    fn lex_errors_report_line_numbers() {
        let src = "";
        let result = lexer(&src).unwrap_err();

        assert_eq!(result.line_number, 1);
    }

    #[test]
    fn lex_errors_report_line_numbers_accurately() {
        let src = "true\n1foo";
        let result = lexer(&src).unwrap_err();

        assert_eq!(result.line_number, 2);
    }
}