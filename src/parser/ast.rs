use pest::Span;

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    String_(String_<'a>),
    Int(Integer<'a>),
    Bool(Bool<'a>),
    Array(Array<'a>),
    GetVar(GetVar<'a>),
    SetVar(Box<SetVar<'a>>),
    Infix(Box<Infix<'a>>),
    Prefix(Box<Prefix<'a>>),
    Index(Box<Index<'a>>),

    If(Box<If<'a>>),
    While(Box<While<'a>>),
    Do(Box<Do<'a>>),
}

#[derive(Debug, Clone)]
pub struct If<'a> {
    pub cond: Expression<'a>,
    pub body: Program<'a>,
    pub other: Program<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct While<'a> {
    pub cond: Expression<'a>,
    pub body: Program<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Do<'a> {
    pub body: Program<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Infix<'a> {
    pub left: Expression<'a>,
    pub operator: &'a str,
    pub right: Expression<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Prefix<'a> {
    pub operator: &'a str,
    pub right: Expression<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct String_<'a> {
    pub value: &'a str,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Integer<'a> {
    pub value: u64,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Bool<'a> {
    pub value: bool,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Array<'a> {
    pub values: Vec<Expression<'a>>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct GetVar<'a> {
    pub name: &'a str,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct SetVar<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Index<'a> {
    pub callee: Expression<'a>,
    pub index: Expression<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    ExprStmt(ExprStmt<'a>),
    DebugPrint(DebugPrint<'a>),
    Break(Break<'a>),
    Next(Next<'a>),
}

#[derive(Debug, Clone)]
pub struct ExprStmt<'a> {
    pub expr: Expression<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct DebugPrint<'a> {
    pub expr: Expression<'a>,
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Break<'a> {
    pub pos: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct Next<'a> {
    pub pos: Span<'a>,
}

pub type Program<'a> = Vec<Statement<'a>>;
