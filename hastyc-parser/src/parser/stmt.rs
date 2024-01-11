use hastyc_common::{identifiers::ASTNodeID, span::Span};

use super::{Attributes, Item};

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
    // TODO: Let binding
    Item(Item),
    /// Expression followed by a semicolon.
    Expr(Box<Expr>),
    /// Expression without semicolon.
    ExprNS(Box<Expr>)
}

/// Kind of expression
#[derive(Debug, Clone)]
pub enum ExprKind {

}