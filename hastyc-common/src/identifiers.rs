use std::{sync::atomic::AtomicU32, collections::{BTreeMap, HashMap}};

use crate::span::Span;

/// Storage that matches symbol id to string.
#[derive(Debug)]
pub struct SymbolStorage {
    counter: IDCounter,
    id_map: BTreeMap<u32, String>,
    inverse_map: HashMap<String, u32>
}

impl SymbolStorage {
    pub fn new() -> Self {
        Self {
            counter: IDCounter::create(),
            id_map: BTreeMap::new(),
            inverse_map: HashMap::new()
        }
    }

    fn register(&mut self, text: &str) -> Symbol {
        let id = self.counter.next();
        self.id_map.insert(id, text.to_string());
        self.inverse_map.insert(text.to_string(), id);
        Symbol(id)
    }

    pub fn get_or_register(&mut self, text: &str) -> Symbol {
        if let Some(id) = self.inverse_map.get(text) {
            Symbol(*id)
        } else {
            self.register(text)
        }
    }

    pub fn text_of(&self, symbol: Symbol) -> Option<&String> {
        self.id_map.get(&symbol.0)
    }
}

/// Single identifier like "Hello", "function_name" or sth like that.
#[derive(Debug, Clone)]
pub struct Ident {
    pub symbol: Symbol,
    pub span: Span
}

impl Ident {
    pub fn new(symbol: Symbol, span: Span) -> Self {
        Self {
            symbol,
            span
        }
    }
}

/// Symbol used for string interning, this holds only id of internal ident
/// for memory optimization purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol(pub(crate) u32);

/// Counter that uses atomic u32 internally. Used for
/// generation of unique identifiers.
#[derive(Debug)]
pub struct IDCounter(AtomicU32);
impl IDCounter {
    pub const fn create() -> Self {
        Self(AtomicU32::new(0))
    }
    pub fn next(&self) -> u32 {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}
impl Clone for IDCounter {
    fn clone(&self) -> Self {
        Self(AtomicU32::new(self.0.load(std::sync::atomic::Ordering::SeqCst)))
    }
}

macro_rules! impl_basic_id {
    ($name:ident) => {
        impl $name {
            const COUNTER: IDCounter = IDCounter::create();

            pub fn new(id: u32) -> Self {
                Self(id)
            }

            pub fn new_unique() -> Self {
                Self(Self::COUNTER.next())
            }
        }
    };
}
macro_rules! impl_from_counter {
    ($name:ident) => {
        impl From<&IDCounter> for $name {
            fn from(counter: &IDCounter) -> Self {
                Self(counter.next())
            }
        }
    };
}

/// ID of package, this is unique for every crate during compilation,
/// but may change between compilations, so It shouldn't be used
/// between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PkgID(pub u32);
impl_basic_id!(PkgID);

/// ID of source file, this is generated as unique for every
/// source file in the current compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceFileID(pub u32);
impl_basic_id!(SourceFileID);

/// ID of node in AST tree. This is unique **ONLY** in package context,
/// and it may occur that this repeats between multiple packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ASTNodeID(pub u32);
impl ASTNodeID {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}
impl_from_counter!(ASTNodeID);