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
    Return,
    Test,
    Not,
    Or,
    Else,
    Plus,
    Minus,
    LessThan,
    Values,
    Every,
    In,
    Colon,
    LessOrEqual,
    GreaterOrEqual,
    // [7d, kh]: specific cards, as typed
    Cards(String),
    Assign,
    Any,
    Most,
    // the `number` declaration keyword
    NumberType,
    // `move take(...)`: a definition players can use as a move
    Move,
    // free text, e.g. the message in check(rule, message)
    Text(String)
}

#[derive(Debug, PartialEq, Clone)]
pub struct SourceToken {
    pub token: Token,
    pub line_number: u32
}