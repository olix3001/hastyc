use hastyc_common::{identifiers::{ASTNodeID, Symbol, Ident}, span::Span, path::Path};

use super::{Attributes, Item, Pat, Ty, Block};

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
    Field(Box<Expr>, Ident),
    Assign(Box<Expr>, Box<Expr>),
    Unary(UnOpKind, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Box<Expr>>),
    /// if expr {block} else {block}
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    Block(Box<Block>),
    Loop(Box<Block>),
    While(Box<Expr>, Box<Block>)
}

#[derive(Debug, Clone)]
pub enum UnOpKind {
    Neg,
    Not
}

pub type BinOp = Spanned<BinOpKind>;
#[derive(Debug, Clone)]
pub enum BinOpKind {
    Add, Sub, Mul,
    Div, Rem, And,
    Or, BitAnd, BitXor,
    BitOr, Shl, Shr,
    Eq, Lt, Le, Ne, Ge, Gt
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

#[derive(Debug, Clone)]
pub struct Spanned<Kind> {
    pub kind: Kind,
    pub span: Span
}

impl<Kind> Spanned<Kind> {
    pub fn new(kind: Kind, span: Span) -> Self {
        Self {
            kind,
            span
        }
    }
}

pub trait MakeSpanned where Self: Sized {
    fn spanned(self, span: Span) -> Spanned<Self> {
        Spanned { kind: self, span: span }
    }
}

impl<Kind> MakeSpanned for Kind {}