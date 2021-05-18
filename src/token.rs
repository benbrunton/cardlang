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
    False
}