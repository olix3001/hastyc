use crate::{identifiers::Ident, span::Span};

/// Path to an item. For example this could be `hello::world::MyStruct`
#[derive(Debug)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub span: Span
}

/// Single path segment representing just one path ident.
#[derive(Debug)]
pub struct PathSegment {
    ident: Ident,
}