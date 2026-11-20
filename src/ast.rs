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
    pub expression: Expression,
    // shown to the player when the check fails, instead of the rule
    pub message: Option<String>
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

// Source-like text for expressions and transfers, used in messages
impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Symbol(s) => write!(f, "{}", s),
            Expression::Number(n) if n.fract() == 0.0 => write!(f, "{}", *n as i64),
            Expression::Number(n) => write!(f, "{}", n),
            Expression::Bool(b) => write!(f, "{}", b),
            Expression::FunctionCall(c) => {
                let arguments: Vec<String> = c.arguments.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", c.name, arguments.join(", "))
            },
            Expression::Comparison(c) => {
                let is = if c.negative { "is not" } else { "is" };
                write!(f, "{} {} {}", c.left, is, c.right)
            },
            Expression::And(a) => write!(f, "{} & {}", a.left, a.right),
            Expression::Or(o) => write!(f, "({} or {})", o.left, o.right),
            Expression::Binary(b) => {
                let op = match b.op {
                    BinaryOp::Add => "+",
                    BinaryOp::Subtract => "-",
                    BinaryOp::LessThan => "<",
                    BinaryOp::GreaterThan => ">"
                };
                write!(f, "{} {} {}", b.left, op, b.right)
            }
        }
    }
}

impl std::fmt::Display for Transfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} > {}", self.from, self.to)?;
        match &self.count {
            None => Ok(()),
            Some(TransferCount::End) => write!(f, " end"),
            Some(TransferCount::Number(n)) => write!(f, " {}", n),
            Some(TransferCount::Taking(cards)) => write!(f, " taking {}", cards)
        }
    }
}
