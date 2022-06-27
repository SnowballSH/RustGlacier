use pest::Span;

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Int(Integer<'a>),
    GetVar(GetVar<'a>),
    SetVar(Box<SetVar<'a>>),
    Infix(Box<Infix<'a>>),
    Prefix(Box<Prefix<'a>>),
    Index(Box<Index<'a>>),

    If(Box<If<'a>>),
}

#[derive(Debug, Clone)]
pub struct If<'a> {
    pub cond: Expression<'a>,
    pub body: Program<'a>,
    pub other: Program<'a>,
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
pub struct Integer<'a> {
    pub value: u64,
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

pub type Program<'a> = Vec<Statement<'a>>;