use crate::token::Token;

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
    pub error_type: LexErrorType
}

impl LexError{
    pub fn new(error_type: LexErrorType) -> LexError {
        LexError{ error_type }
    }
}

pub fn lexer(source: &str) -> Result<Vec<Token>, LexError> {
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
                partial_token = None;
                tokens.push(t);
            },
            TokenResult::PartialToken(s) => {
                partial_token = Some(s);
            },
            TokenResult::Empty => {
                partial_token = None;
            },
            TokenResult::Error => {
                let lex_error = LexError::new(LexErrorType::ParseError);
                return Err(lex_error);
            }
        }
    }

    if tokens.len() == 0 {
        let lex_error = LexError::new(LexErrorType::EmptySpecification);
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
        _ => None
    }
}

fn resolve_partial(partial_token: String) -> TokenResult {
    let mut chars = partial_token.chars();
    let first = chars.next().expect("unable to find first char in partial token");
    match first {
        'A'..='z' => TokenResult::Token(Token::Symbol(partial_token)),
        '.' => {
            match chars.last() {
                Some(')') => TokenResult::Empty,
                _ => TokenResult::PartialToken(partial_token)
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
        Some('A'..='z') | Some('_') | Some('0'..='9')=> {
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

        assert_eq!(result[0], Token::Name);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn it_recognises_the_stack_token() {
        let src = "stack";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0], Token::Stack);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn it_recognises_the_deck_keyword() {
        let src = "deck";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0], Token::Deck);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn it_recognises_keywords(){
        let src = "deck players stack current_player define";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0], Token::Deck);
        assert_eq!(result[1], Token::Players);
        assert_eq!(result[2], Token::Stack);
        assert_eq!(result[3], Token::CurrentPlayer);
        assert_eq!(result[4], Token::Define);
    }

    #[test]
    fn it_recognises_an_arbitrary_symbol() {
        let src = "scopa";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0], Token::Symbol("scopa".to_owned()));
    }

    #[test]
    fn it_recognises_camel_case() {
        let src = "StandardDeck";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0], Token::Symbol("StandardDeck".to_owned()));
    }

    #[test]
    fn it_handles_full_lines() {
        let src = "deck StandardDeck";
        let result = lexer(&src).unwrap();
        let expected = vec!(
            Token::Deck,
            Token::Symbol("StandardDeck".to_owned())
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_open_parens() {
        let src = "(";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::OpenParens);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_close_parens() {
        let src = ")";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::CloseParens);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_comma() {
        let src = ",";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::Comma);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_brackets() {
        let src = "{}";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::OpenBracket, Token::CloseBracket);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_transfer() {
        let src = ">";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::Transfer);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_check_and_is() {
        let src = "check cards is fun";
        let result = lexer(&src).unwrap();
        let expected = vec!(
            Token::Check, Token::Symbol("cards".to_owned()),
            Token::Is, Token::Symbol("fun".to_owned())
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_if(){
        let src ="if";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::If);
        assert_eq!(result, expected);
    }

    #[test]
    fn it_handles_numbers(){
        let src ="1";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::Number(1.0));
        assert_eq!(result, expected);
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

        assert_eq!(result[0], Token::Symbol("foo1".to_owned()));
    }

    #[test]
    fn newlines_are_tokenised() {
        let src = "\n";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0], Token::Newline);
    }

    #[test]
    fn it_ignores_comments() {
        let src = "name .( this is a comment ) test1";
        let result = lexer(&src).unwrap();
        let expected = vec!(Token::Name, Token::Symbol("test1".to_owned()));
        assert_eq!(result, expected);
    }
}