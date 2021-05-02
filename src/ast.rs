#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Declaration(Declaration),
    Definition(Definition),
    Transfer(Transfer)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Symbol(String),
    Number(f64)
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
pub struct Transfer {
    pub from: String,
    pub to: String,
    pub modifier: Option<TransferModifier>,
    pub count: Option<TransferCount>
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransferModifier {
    Alternate
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransferCount {
    End
}