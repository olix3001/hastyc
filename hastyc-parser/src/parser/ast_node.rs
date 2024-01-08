use std::sync::Arc;

use hastyc_common::{identifiers::{ASTNodeID, IDCounter}, span::Span};

/// Currently unimplemented, basically there for future implementation.
#[derive(Debug, Default)]
pub struct Attributes { }

/// Source package, this is basically a root node for the whole AST.
pub struct Package {
    pub attrs: Attributes,
    pub items: ItemStream,
    pub id: ASTNodeID,
    pub idgen: IDCounter
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
    Module(ModuleKind)
}

/// Kind of module as modules can be defined in separate files or inlined.
#[derive(Debug)]
pub enum ModuleKind {
    /// Inline module.
    Inlined(ItemStream),
    /// Module defined in an external file which is already loaded.
    External(ItemStream),
    /// Module defined in an external file which is not yet loaded.
    Unloaded
}