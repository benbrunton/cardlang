use crate::token::Token;

enum TokenResult {
    Token(Token),
    PartialToken(String),
    Empty
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexErrorType{
    EmptySpecification
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
            if current_char == ' ' {
                TokenResult::Empty
            } else {
                handle_partial(current_char.to_string(), next_char)
            }
        },
        Some(x) => {
            let new_partial = format!("{}{}", x, current_char);
            handle_partial(new_partial, next_char)
        }
    }
}

fn handle_partial(current_partial: String, next_char: Option<&char>) -> TokenResult {
    let keyword_result = handle_keyword(&current_partial, next_char);

    if keyword_result.is_some() {
        return keyword_result.expect("should be a keyword");
    }

    if is_word_finished(next_char) {
        return TokenResult::Token(Token::Symbol(current_partial));
    }

    TokenResult::PartialToken(current_partial)
}

fn handle_keyword(partial_token: &str, next_char: Option<&char>) -> Option<TokenResult> {
    if !is_word_finished(next_char) {
        return None
    }

    match partial_token {
        "name" => Some(TokenResult::Token(Token::Name)),
        "set" => Some(TokenResult::Token(Token::Set)),
        "as" => Some(TokenResult::Token(Token::As)),
        "stack" => Some(TokenResult::Token(Token::Stack)),
        _ => None
    }
}

fn is_word_finished(next_char: Option<&char>) -> bool {
    match next_char {
        Some('A'..='z') | Some('_') => {
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
    fn it_recognises_the_set_token() {
        let src = "set";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0], Token::Set);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn it_recognises_the_as_token() {
        let src = "as";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0], Token::As);
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
        let src = "set deck as StandardDeck";
        let result = lexer(&src).unwrap();
        let expected = vec!(
            Token::Set,
            Token::Symbol("deck".to_owned()),
            Token::As,
            Token::Symbol("StandardDeck".to_owned())
        );
        assert_eq!(result, expected);
    }
}