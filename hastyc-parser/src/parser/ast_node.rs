use std::sync::Arc;

use hastyc_common::{identifiers::{ASTNodeID, IDCounter, Ident, SymbolStorage}, span::Span, path::Path};

/// Currently unimplemented, basically there for future implementation.
#[derive(Debug, Default)]
pub struct Attributes { }

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
#[derive(Debug)]
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
#[derive(Debug)]
pub enum ItemKind {
    Module(ItemStream),
    Import(ImportKind, ImportTree)
}

impl ItemKind {
    pub fn name_of_type(&self) -> &'static str {
        match self {
            Self::Module(_) => "Module",
            Self::Import(_, _) => "Import"
        }
    }
}

/// Imports can be either relative (eg. `import hello::world`),
/// super (eg. `import super::hello`), or package based (eg. `import pkg::hello`).
#[derive(Debug)]
pub enum ImportKind {
    Relative,
    Super,
    Package
}

/// As Hasty uses import system inspired by Rust, imports are not paths,
/// but trees. For example `import a::{b, c::{self, d}}` will produce a tree.
#[derive(Debug)]
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

#[derive(Debug)]
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