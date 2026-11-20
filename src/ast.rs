#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Declaration(Declaration),
    Definition(Definition),
    Transfer(Transfer),
    FunctionCall(FunctionCall),
    IfStatement(IfStatement),
    CheckStatement(CheckStatement),
    ReturnStatement(ReturnStatement),
    // `values { Jack 8, Queen 9 }`: card values by rank
    ValueTable(Vec<(String, f64)>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Symbol(String),
    Number(f64),
    Comparison(Box<Comparison>),
    Bool(bool),
    FunctionCall(FunctionCall),
    And(Box<And>),
    Or(Box<Or>),
    Binary(Box<Binary>)
}

#[derive(Debug, PartialEq, Clone)]
pub enum GlobalKey {
    Name,
    Players,
    Stack,
    Deck,
    CurrentPlayer,
    // a stack that every player has, e.g. `stack collection per player`
    PlayerStack
}

#[derive(Debug, PartialEq, Clone)]
pub struct Declaration {
    pub key: GlobalKey,
    pub value: Expression
}

#[derive(Debug, PartialEq, Clone)]
pub struct Definition {
    pub name: String,
    pub arguments: Vec<String>,
    pub body: Vec<Statement>
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub expression: Expression,
    pub body: Vec<Statement>,
    pub else_body: Vec<Statement>
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
    // every card in the source
    End,
    // this many cards to each destination stack
    Number(usize),
    // exactly these cards, all or nothing
    Taking(Expression)
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Comparison {
    pub left: Expression,
    pub right: Expression,
    pub negative: bool
}

#[derive(Debug, PartialEq, Clone)]
pub struct CheckStatement {
    pub expression: Expression
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStatement {
    pub expression: Expression
}

#[derive(Debug, PartialEq, Clone)]
pub struct And {
    pub left: Expression,
    pub right: Expression
}

#[derive(Debug, PartialEq, Clone)]
pub struct Or {
    pub left: Expression,
    pub right: Expression
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Subtract,
    LessThan,
    GreaterThan
}

#[derive(Debug, PartialEq, Clone)]
pub struct Binary {
    pub op: BinaryOp,
    pub left: Expression,
    pub right: Expression
}

pub const RANK_NAMES: [&str; 13] = [
    "Ace", "Two", "Three", "Four", "Five", "Six", "Seven",
    "Eight", "Nine", "Ten", "Jack", "Queen", "King"
];
