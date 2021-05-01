#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Name,
    Symbol(String),
    Set,
    As,
    Stack
}