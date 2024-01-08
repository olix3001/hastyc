use crate::identifiers::Ident;

/// Path to an item. For example this could be `hello::world::MyStruct`
pub struct Path {
    pub segments: Vec<PathSegment>
}

/// Single path segment representing just one path ident.
pub struct PathSegment {
    ident: Ident,
}