#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Name,
    Symbol(String),
    Number(f64),
    Stack,
    Deck,
    Players,
    CurrentPlayer,
    Define,
    OpenParens,
    CloseParens,
    Comma,
    OpenBracket,
    CloseBracket,
    Transfer,
    Check,
    Is,
    If,
    Newline,
    True,
    False,
    Ampersand,
    Return
}

#[derive(Debug, PartialEq, Clone)]
pub struct SourceToken {
    pub token: Token,
    pub line_number: u32
}