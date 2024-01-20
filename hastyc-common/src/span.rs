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

    pub fn dummy() -> Self {
        Self {
            start: 0,
            end: 0,
            source: SourceFileID(0)
        }
    }

    pub fn from_begin_end(begin: Span, end: Span) -> Self {
        Self {
            start: begin.start,
            end: end.end,
            source: begin.source
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

    /// Converts span to relative start, eg. (line, col)
    pub fn to_relative(&self, source: &SourceFile) -> (u32, u32) {
        let mut line = 0;
        let mut col = 0;

        for (i, char) in source.src.as_ref().unwrap().chars().enumerate() {
            if char == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }

            if i == self.start as usize {
                return (line + 1, col)
            }
        }
        return (0, 0)
    }

    fn get_line_start_end(source: &SourceFile, line: u32) -> (u32, u32) {
        let mut start = 0;
        let mut cline = 0;
        
        for (i, char) in source.src.as_ref().unwrap().chars().enumerate() {
            if char == '\n' {
                if cline + 1 == line {
                    return (start, i as u32)
                }
                start = i as u32;
                cline += 1;
            }
        }

        return (0, 0)
    }

    /// This returns (line_text, line_start_span)
    pub fn get_line(&self, source: &SourceFile) -> (String, u32) {
        let relative = self.to_relative(source);
        let (line_start, line_end) = Self::get_line_start_end(source, relative.0);

        let line = source.get_span(
            &Span::new(source.id, line_start + 1, line_end)
        );

        (line, self.start - line_start - 1)
    }
}