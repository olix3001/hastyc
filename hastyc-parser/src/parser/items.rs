use std::sync::Arc;

use hastyc_common::{identifiers::{ASTNodeID, IDCounter, Ident, SymbolStorage}, span::Span, path::Path};

use super::StmtStream;

/// Currently unimplemented, basically there for future implementation.
#[derive(Debug, Clone)]
pub struct Attributes {
    pub attributes: Vec<Attribute>
}

impl Attributes {
    pub fn empty() -> Self {
        Self {
            attributes: Vec::new()
        }
    }
}

/// One single attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub ident: Ident,
    pub kind: AttributeKind
}

#[derive(Debug, Clone)]
pub enum AttributeKind {
    /// Attribute without any additional data like `#[hello]`
    FlagAttribute,
    // TODO: Add more attribute kinds when necessary
}

/// Source package, this is basically a root node for the whole AST.
#[derive(Debug)]
pub struct Package {
    pub attrs: Attributes,
    pub items: ItemStream,
    pub id: ASTNodeID,
    pub idgen: IDCounter,
    pub symbol_storage: SymbolStorage
}

/// Stream of language items.
#[derive(Debug, Clone)]
pub struct ItemStream {
    pub items: Arc<Vec<Item>>
}

impl ItemStream {
    pub fn empty() -> Self {
        Self {
            items: Arc::new(Vec::new())
        }
    }
    pub fn from_items(items: Vec<Item>) -> Self {
        Self {
            items: Arc::new(items)
        }
    }
}

/// Single language item, it hold its kind, attributes, id and more useful information.
#[derive(Debug, Clone)]
pub struct Item {
    pub attrs: Attributes,
    pub id: ASTNodeID,
    pub visibility: Visibility,
    pub kind: ItemKind,
    pub ident: Ident,
    pub span: Span
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Inherited
}

/// Kind of language item. These are things like imports, function declarations,
/// struct definitions, constants, etc...
#[derive(Debug, Clone)]
pub enum ItemKind {
    Module(ItemStream),
    Import(ImportKind, ImportTree),
    Fn(Function)
}

impl ItemKind {
    pub fn name_of_type(&self) -> &'static str {
        match self {
            Self::Module(_) => "Module",
            Self::Import(_, _) => "Import",
            Self::Fn(_) => "Function"
        }
    }
}

/// Imports can be either relative (eg. `import hello::world`),
/// super (eg. `import super::hello`), or package based (eg. `import pkg::hello`).
#[derive(Debug, Clone, Copy)]
pub enum ImportKind {
    Relative,
    Super,
    Package
}

/// As Hasty uses import system inspired by Rust, imports are not paths,
/// but trees. For example `import a::{b, c::{self, d}}` will produce a tree.
#[derive(Debug, Clone)]
pub struct ImportTree {
    pub prefix: Path,
    pub kind: ImportTreeKind,
    pub span: Span
}

impl ImportTree {
    /// Import tree with only prefix, name and span
    pub fn simple(mut name: Path, span: Span) -> Self {
        let import_name = name.pop();
        Self {
            prefix: name,
            kind: ImportTreeKind::Simple(import_name.unwrap().into()),
            span
        }
    }

    /// Import tree with * import
    pub fn glob(name: Path, span: Span) -> Self {
        Self {
            prefix: name,
            kind: ImportTreeKind::Glob,
            span
        }
    }

    /// Nested import
    pub fn nested(prefix: Path, imports: Vec<(ImportTree, ASTNodeID)>, span: Span) -> Self {
        Self {
            prefix,
            kind: ImportTreeKind::Nested(imports),
            span
        }
    }

    /// Self import
    pub fn self_import(prefix: Path, span: Span) -> Self {
        Self {
            prefix,
            kind: ImportTreeKind::SelfImport,
            span
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImportTreeKind {
    /// Import prefix
    Simple(Ident),
    /// Self import
    SelfImport,
    /// import prefix::{ ... }
    Nested(Vec<(ImportTree, ASTNodeID)>),
    /// import prefix::*
    Glob
}

/// Function definition.
#[derive(Debug, Clone)]
pub struct Function {
    pub generics: Generics,
    pub signature: FnSignature,
    pub body: Option<Box<Block>>
}

/// Block of code like `{ ... }` in `fn hello() { ... }`.
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: StmtStream,
    pub id: ASTNodeID,
    pub span: Span,
}

impl Block {
    pub fn empty() -> Self {
        Self {
            stmts: StmtStream::empty(),
            id: ASTNodeID(0),
            span: Span::dummy()
        }
    }
}

/// Generics. These are those `<T>` thingies.
#[derive(Debug, Clone)]
pub struct Generics {
    // TODO: Implement generics in some reasonable way.
}

/// Function signature containing information about its types
/// and things like this.
#[derive(Debug, Clone)]
pub struct FnSignature {
    pub is_const: bool,
    pub is_async: bool,
    pub inputs: Vec<FnInput>,
    pub output: FnRetTy, 
    pub span: Span   
}


/// Function input param.
#[derive(Debug, Clone)]
pub struct FnInput {
    pub attributes: Attributes,
    pub id: ASTNodeID,
    pub span: Span,
    pub pat: Pat,
    pub ty: Ty
}

#[derive(Debug, Clone)]
pub enum FnRetTy {
    Default, // This is () for normal functions.
    Ty(Ty)
}

/// Simple type like `i32`, `()` or more complex one like
/// `hello::world::MyType`.
#[derive(Debug, Clone)]
pub struct Ty {
    pub id: ASTNodeID,
    pub kind: TyKind,
    pub span: Span
}

/// Kind of type.
#[derive(Debug, Clone)]
pub enum TyKind {
    /// This is used for passing "self" to the function as an argument.
    SelfTy,
    /// Anything like `i32` or `hello::Type` falls into this category.
    Path(Path),
    /// Void type defined by `()`.
    Void,
    /// Something with an infinite loop that should NEVER return.
    Never,
    /// Unkown type, should be infered.
    Infer
}

/// A pattern.
#[derive(Debug, Clone)]
pub struct Pat {
    pub id: ASTNodeID,
    pub kind: PatKind,
    pub span: Span
}

/// Kind of pattern.
#[derive(Debug, Clone)]
pub enum PatKind {
    SelfPat,
    Ident(Ident)
}