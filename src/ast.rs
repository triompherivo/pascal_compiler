// src/ast.rs

#[derive(Debug, PartialEq, Clone)]
pub enum PascalType {
    Integer,
    Real,
    String,
    Boolean,
    RecordType(Vec<(String, PascalType)>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(i64),
    Real(f64),
    StringLiteral(String),
    Variable(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    RecordLiteral(Vec<(String, Expr)>),
    FieldAccess {
        record: Box<Expr>,
        field: String,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CaseBranch {
    pub label: Expr,
    pub stmt: Statement,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ForDirection {
    To,
    DownTo,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assignment { variable: String, expr: Expr },
    Compound(Vec<Statement>),
    If { condition: Expr, then_branch: Box<Statement>, else_branch: Option<Box<Statement>> },
    While { condition: Expr, body: Box<Statement> },
    For { 
        variable: String,
        start: Box<Expr>,
        end: Box<Expr>,
        direction: ForDirection,
        body: Box<Statement>,
    },
    RepeatUntil { body: Box<Statement>, condition: Expr },
    FunctionDecl { name: String, params: Vec<(String, PascalType)>, body: Box<Statement> },
    ProcedureDecl { name: String, params: Vec<(String, PascalType)>, body: Box<Statement> },
    ProcedureCall { name: String, args: Vec<Expr> },
    Case { expr: Expr, branches: Vec<CaseBranch>, else_branch: Option<Box<Statement>> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Declaration {
    VarDeclaration { names: Vec<String>, typ: PascalType },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub main: Statement,
}
