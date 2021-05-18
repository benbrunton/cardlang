#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Declaration(Declaration),
    Definition(Definition),
    Transfer(Transfer),
    FunctionCall(FunctionCall),
    IfStatement(IfStatement)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Symbol(String),
    Number(f64),
    Comparison(Box<Comparison>),
    Bool(bool),
    FunctionCall(FunctionCall)
}

impl Expression {
    pub fn to_number(&self) -> f64 {
        match self {
            Self::Number(n) => *n,
            _ => 0.0
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum GlobalKey {
    Name,
    Players,
    Stack,
    Deck,
    CurrentPlayer
}

#[derive(Debug, PartialEq, Clone)]
pub struct Declaration {
    pub key: GlobalKey,
    pub value: Expression
}

#[derive(Debug, PartialEq, Clone)]
pub struct Definition {
    pub name: String,
    pub body: Vec<Statement>
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub expression: Expression,
    pub body: Vec<Statement>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Transfer {
    pub from: String,
    pub to: String,
    pub modifier: Option<TransferModifier>,
    pub count: Option<TransferCount>
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransferModifier {
    //Alternate
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransferCount {
    End
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Comparison {
    pub left: Expression,
    pub right: Expression
}