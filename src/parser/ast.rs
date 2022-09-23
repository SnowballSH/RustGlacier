use pest::Span;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AstSpan {
    pub start: usize,
    pub end: usize,
}

impl<'a> From<Span<'a>> for AstSpan {
    fn from(span: Span) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    String_(String_<'a>),
    Int(Integer),
    Bool(Bool),
    Array(Array<'a>),
    GetVar(GetVar<'a>),
    SetVar(Box<SetVar<'a>>),
    Infix(Box<Infix<'a>>),
    Prefix(Box<Prefix<'a>>),
    Index(Box<Index<'a>>),

    PointerAssign(Box<PointerAssign<'a>>),

    If(Box<If<'a>>),
    While(Box<While<'a>>),
    Do(Box<Do<'a>>),
}

#[derive(Debug, Clone)]
pub struct If<'a> {
    pub cond: Expression<'a>,
    pub body: Program<'a>,
    pub other: Program<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct While<'a> {
    pub cond: Expression<'a>,
    pub body: Program<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Do<'a> {
    pub body: Program<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Infix<'a> {
    pub left: Expression<'a>,
    pub operator: &'a str,
    pub right: Expression<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Prefix<'a> {
    pub operator: &'a str,
    pub right: Expression<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct String_<'a> {
    pub value: &'a str,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Integer {
    pub value: u64,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Bool {
    pub value: bool,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Array<'a> {
    pub values: Vec<Expression<'a>>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct GetVar<'a> {
    pub name: &'a str,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct SetVar<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Index<'a> {
    pub callee: Expression<'a>,
    pub index: Expression<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct PointerAssign<'a> {
    pub ptr: Expression<'a>,
    pub value: Expression<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    ExprStmt(ExprStmt<'a>),
    DebugPrint(DebugPrint<'a>),
    Break(Break),
    Next(Next),
}

#[derive(Debug, Clone)]
pub struct ExprStmt<'a> {
    pub expr: Expression<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct DebugPrint<'a> {
    pub expr: Expression<'a>,
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Break {
    pub pos: AstSpan,
}

#[derive(Debug, Clone)]
pub struct Next {
    pub pos: AstSpan,
}

pub type Program<'a> = Vec<Statement<'a>>;
