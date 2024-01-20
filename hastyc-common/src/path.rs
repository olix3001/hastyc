use crate::{identifiers::Ident, span::Span};

/// Path to an item. For example this could be `hello::world::MyStruct`
#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub span: Span
}

impl Path {
    pub fn empty() -> Self {
        Self {
            segments: Vec::new(),
            span: Span::dummy()
        }
    }

    pub fn pop(&mut self) -> Option<PathSegment> {
        self.segments.pop()
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn shifted_clone(&self, count: u32) -> Path {
        let mut new_segments = Vec::new();
        for seg in self.segments.clone().into_iter().skip(count as usize) {
            new_segments.push(seg)
        }
        let start = new_segments.first().unwrap().ident.span;
        let end = new_segments.last().unwrap().ident.span;
        Path {
            segments: new_segments,
            span: Span::from_begin_end(start, end)
        }
    }
}

/// Single path segment representing just one path ident.
#[derive(Debug, Clone)]
pub struct PathSegment {
    pub ident: Ident
}

impl PathSegment {
    pub fn new(ident: Ident) -> Self {
        Self {
            ident
        }
    }
}

impl Into<Ident> for PathSegment {
    fn into(self) -> Ident {
        self.ident
    }
}