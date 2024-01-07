use std::fmt::Debug;

use crate::{identifiers::SourceFileID, source::SourceFile};

/// Span represents region in the source code from which
/// given data come.
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub source: SourceFileID
}

impl Span {
    pub fn new(
        source: SourceFileID,
        start: u32,
        end: u32
    ) -> Self {
        Self {
            start,
            end,
            source
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Get text from source file, this checks whether
    /// source is same as expected source, returning None
    /// if it isn't.
    pub fn get_text(&self, source: &SourceFile) -> Option<String> {
        if self.source != source.id { return None }
        if let Some(ref src) = source.src {
            Some(
                src.chars()
                    .skip(self.start as usize)
                    .take(self.len() as usize)
                    .collect()
                )
        } else { None }
    }
}