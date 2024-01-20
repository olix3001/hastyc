use std::{fmt::Debug, path::PathBuf};

use crate::{identifiers::{PkgID, SourceFileID}, span::Span};

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

impl std::fmt::Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RawText => f.write_str("<raw source>"),
            Self::LocalPath(path) => f.write_str(
                // TODO: make this relative to project root instead of canonical
                PathBuf::from(path).canonicalize().unwrap().to_str().unwrap()
            )
        }
    }
}

impl SourceFile {
    /// Creates new source file from raw text, this is
    /// useful for testing.
    pub fn new_raw(text: String, pkg: PkgID, id: SourceFileID) -> Self {
        let len = text.len();
        Self {
            name: FileName::RawText,
            src: Some(text),
            clen: len,
            pkg,
            id
        }
    }

    /// Get span from the file
    pub fn get_span(&self, span: &Span) -> String {
        if let Some(ref src) = self.src {
            src.chars()
                .skip(span.start as usize)
                .take((span.end - span.start) as usize)
                .collect()
        } else {
            unimplemented!("Getting span of sources without loaded source is unimplemented")
        }
    }
}