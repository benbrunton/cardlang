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
    ParseError,
    UnterminatedComment
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

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self.error_type {
            LexErrorType::EmptySpecification => "the file is empty",
            LexErrorType::ParseError => "unrecognised character (for example '\"' or a decimal number)",
            LexErrorType::UnterminatedComment => "a comment is never closed - missing ')'"
        };
        write!(f, "line {}: {}", self.line_number, message)
    }
}

pub fn lexer(source: &str) -> Result<Vec<SourceToken>, LexError> {
    let mut line_number = 1;
    let mut tokens = vec!();
    let mut chars = source.chars().peekable();

    let mut partial_token: Option<String> = None;

    while let Some(current_char) = chars.next() {

        // newlines inside a comment still count as lines
        if current_char == '\n' && partial_token.is_some() {
            line_number += 1;
        }

        let next_char = chars.peek();
        let result = handle_char(&partial_token, current_char, next_char);

        match result {
            TokenResult::Token(t) => {
                let is_newline = t == Token::Newline;

                partial_token = None;
                let source_token = SourceToken{
                    token: t,
                    line_number
                };
                tokens.push(source_token);

                // a newline belongs to the line it ends
                if is_newline {
                    line_number += 1;
                }
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

    if partial_token.is_some() {
        return Err(LexError::new(LexErrorType::UnterminatedComment, line_number));
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
            match handle_single_chars(current_char) {
                Some(result) => result,
                None => handle_partial(current_char.to_string(), next_char)
            }
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
        ' ' | '\t' | '\r' => Some(TokenResult::Empty),
        ',' => Some(TokenResult::Token(Token::Comma)),
        '{' => Some(TokenResult::Token(Token::OpenBracket)),
        '}' => Some(TokenResult::Token(Token::CloseBracket)),
        '>' => Some(TokenResult::Token(Token::Transfer)),
        '\n' => Some(TokenResult::Token(Token::Newline)),
        '.' => Some(TokenResult::PartialToken(current_char.to_string())),
        '&' => Some(TokenResult::Token(Token::Ampersand)),
        '+' => Some(TokenResult::Token(Token::Plus)),
        '-' => Some(TokenResult::Token(Token::Minus)),
        '<' => Some(TokenResult::Token(Token::LessThan)),
        _ => None
    }
}

fn handle_partial(current_partial: String, next_char: Option<&char>) -> TokenResult {
    if let Some(keyword) = handle_keyword(&current_partial, next_char) {
        return keyword;
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
        "def" => Some(TokenResult::Token(Token::Define)),
        "check" => Some(TokenResult::Token(Token::Check)),
        "is" => Some(TokenResult::Token(Token::Is)),
        "if" => Some(TokenResult::Token(Token::If)),
        "true" => Some(TokenResult::Token(Token::True)),
        "false" => Some(TokenResult::Token(Token::False)),
        "return" => Some(TokenResult::Token(Token::Return)),
        ".test" => Some(TokenResult::Token(Token::Test)),
        "not" => Some(TokenResult::Token(Token::Not)),
        "or" => Some(TokenResult::Token(Token::Or)),
        "else" => Some(TokenResult::Token(Token::Else)),
        "values" => Some(TokenResult::Token(Token::Values)),
        _ => None
    }
}

fn resolve_partial(partial_token: String) -> TokenResult {
    let mut chars = partial_token.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return TokenResult::Error
    };
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
        let src = "deck players stack current_player def";
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

    #[test]
    fn it_recognises_the_ampersand() {
        let src = "&";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Ampersand);
    }

    #[test]
    fn it_recognises_the_return_keyword() {
        let src = "return";
        let result = lexer(&src).unwrap();

        assert_eq!(result[0].token, Token::Return);
    }


    #[test]
    fn it_recognises_tests() {
        let src = ".test()";
        let result = lexer(&src).unwrap();
        let expected = vec!(
            Token::Test,
            Token::OpenParens,
            Token::CloseParens
        );

        assert_eq!(result[0].token, expected[0]);
        assert_eq!(result[1].token, expected[1]);
        assert_eq!(result[2].token, expected[2]);
    }

    #[test]
    fn it_recognises_not() {
        let src = "not";
        let result = lexer(&src).unwrap();
        let expected = vec!(
            Token::Not,
        );

        assert_eq!(result[0].token, expected[0]);
    }

    #[test]
    fn tabs_and_carriage_returns_are_whitespace() {
        let result = lexer("name\tx\r\nplayers 2\r\n").unwrap();
        let tokens: Vec<Token> = result.iter().map(|t| t.token.clone()).collect();

        assert_eq!(tokens, vec!(
            Token::Name, Token::Symbol("x".to_string()), Token::Newline,
            Token::Players, Token::Number(2.0), Token::Newline
        ));
    }

    #[test]
    fn newline_tokens_belong_to_the_line_they_end() {
        let result = lexer("name x\nplayers 2").unwrap();

        assert_eq!(result[2].token, Token::Newline);
        assert_eq!(result[2].line_number, 1);
        assert_eq!(result[3].line_number, 2);
    }

    #[test]
    fn lines_inside_comments_are_counted() {
        let result = lexer(".(one\ntwo\nthree)\nname x\n1foo").unwrap_err();

        assert_eq!(result.line_number, 5);
    }

    #[test]
    fn an_unterminated_comment_is_an_error() {
        let result = lexer("name x\n.(oops\nplayers 2").unwrap_err();

        assert_eq!(result.error_type, LexErrorType::UnterminatedComment);
    }
}
