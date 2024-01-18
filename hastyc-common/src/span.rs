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
        let mut start = 0;
        for (line_no, line_start) in source.lines.iter().enumerate() {
            if *line_start > self.start {
                line = line_no;
                break;
            }
            start = *line_start
        }

        if line > 0 {
            line = line - 1;
        }

        return (line as u32, self.start - start)
    }

    /// This returns (line_text, line_start_span)
    pub fn get_line(&self, source: &SourceFile) -> (String, u32) {
        let relative = self.to_relative(source);
        let line_start = source.lines.get(relative.0 as usize).unwrap();
        let line_end = source.lines.get(relative.0 as usize + 1);

        let line = source.get_span(
            &Span::new(source.id, *line_start + 1, match line_end {
                Some(end) => *end,
                None => source.clen as u32
            })
        );

        (line, self.start - *line_start - 1)
    }
}