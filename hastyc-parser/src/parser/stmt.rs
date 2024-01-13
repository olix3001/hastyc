use hastyc_common::{identifiers::{ASTNodeID, Symbol, Ident}, span::Span, path::Path};

use super::{Attributes, Item, Pat, Ty};

/// Stream of statements. This is like a part of code.
#[derive(Debug, Clone)]
pub struct StmtStream {
    pub stmts: Vec<Stmt>
}

impl StmtStream {
    pub fn empty() -> Self {
        Self {
            stmts: Vec::new()
        }
    }

    pub fn from_vec(v: Vec<Stmt>) -> Self {
        Self {
            stmts: v
        }
    }
}

/// One single statement, this can be variable declaration,
/// function call, some conditional flow or things like that.
#[derive(Debug, Clone)]
pub struct Stmt {
    pub id: ASTNodeID,
    pub kind: StmtKind,
    pub span: Span
}

/// Expression is like a statement with return value.
#[derive(Debug, Clone)]
pub struct Expr {
    pub id: ASTNodeID,
    pub kind: ExprKind,
    pub span: Span,
    pub attrs: Attributes,
}

/// Kind of statement
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Let statement like `let _: _ = _;`.
    LetBinding(Box<LetBinding>),
    Item(Box<Item>),
    /// Expression followed by a semicolon.
    Expr(Box<Expr>),
    /// Expression without semicolon.
    ExprNS(Box<Expr>)
}

/// Kind of expression
#[derive(Debug, Clone)]
pub enum ExprKind {
    Path(Path),
    Literal(Lit),
    /// Field access like `value.field`
    Field(Box<Expr>, Ident)
}

#[derive(Debug, Clone)]
pub struct LetBinding {
    pub id: ASTNodeID,
    pub pat: Pat,
    pub ty: Option<Ty>,
    pub kind: LetBindingKind,
    pub span: Span,
    pub attribs: Attributes
}

#[derive(Debug, Clone)]
pub enum LetBindingKind {
    /// Just variable declaration `let variable;`
    Decl,
    /// Variable declaration with assignment `let variable = value;`
    Init(Box<Expr>)
}

#[derive(Debug, Clone)]
pub struct Lit {
    pub id: ASTNodeID,
    pub kind: LitKind,
    pub symbol: Symbol
}

#[derive(Debug, Clone)]
pub enum LitKind {
    Bool,
    Char,
    Integer,
    Float,
    String
}