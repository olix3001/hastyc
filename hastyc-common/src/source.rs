use std::fmt::Debug;

use crate::identifiers::{PkgID, SourceFileID};

/// Source file mapping. This is used for keeping track of
/// where does specified part of the source code come from.
#[derive(Clone)]
pub struct SourceFile {
    /// Path/name of the file from where this is.
    pub name: FileName,
    /// Full source code of the given file.
    pub src: Option<String>,
    /// Length of the source code in characters.
    pub clen: usize,
    /// Markers of line beginnings in the source file.
    pub lines: Vec<u32>,
    /// Package associated with this source file.
    pub pkg: PkgID,
    /// ID associated with this source.
    pub id: SourceFileID
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("SourceFile")
            .field("name", &self.name)
            .field("src", &self.src)
            .field("pkg", &self.pkg)
            .field("id", &self.id)
            .finish()
    }
}

/// As file names may come from multiple sources, like
/// current crate, dependency, or something internal, we
/// need enum to keep track of this.
#[derive(Debug, Clone)]
pub enum FileName {
    LocalPath(String),
    RawText
}

impl SourceFile {
    fn calculate_lines(text: &str) -> Vec<u32> {
        let mut lines = Vec::new();

        for (i, c) in text.chars().enumerate() {
            if c == '\n' {
                lines.push(i as u32);
            }
        }

        lines
    }

    /// Creates new source file from raw text, this is
    /// useful for testing.
    pub fn new_raw(text: String, pkg: PkgID, id: SourceFileID) -> Self {
        let len = text.len();
        let lines = Self::calculate_lines(&text);
        Self {
            name: FileName::RawText,
            src: Some(text),
            clen: len,
            lines: lines,
            pkg,
            id
        }
    }
}