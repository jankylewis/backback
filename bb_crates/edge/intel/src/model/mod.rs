//! Values the API hands out: paths, URIs, spans, locations.

pub(crate) mod bb_doc;
pub(crate) mod results;
pub(crate) mod store;
pub(crate) mod unique;

use std::path::{Path, PathBuf};

use bb_text::LspPoint;
use url::Url;

use crate::IntelError;

/// An absolute path to a document on disk.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocPath(PathBuf);

impl DocPath {
    /// Wrap an absolute path.
    ///
    /// # Errors
    /// [`IntelError::BadPath`] when `path` is relative.
    pub fn new(path: PathBuf) -> Result<Self, IntelError> {
        if path.is_absolute() {
            Ok(Self(path))
        } else {
            Err(IntelError::BadPath(path))
        }
    }

    /// The path.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// The `file:` URI the server knows this document by.
    pub(crate) fn to_url(&self) -> Result<Url, IntelError> {
        Url::from_file_path(&self.0).map_err(|(): ()| IntelError::BadPath(self.0.clone()))
    }
}

/// Where a server answer points: a file on disk, or a document only the
/// server has (decompiled metadata, generated source).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DocUri {
    /// A file on disk.
    File(DocPath),
    /// Any other URI, kept as text.
    Virtual(String),
}

impl DocUri {
    /// Classify a server URI.
    pub(crate) fn from_url(url: &Url) -> Self {
        let file: Option<DocPath> = if url.scheme() == "file" {
            url.to_file_path()
                .ok()
                .and_then(|p: PathBuf| DocPath::new(p).ok())
        } else {
            None
        };
        file.map_or_else(|| Self::Virtual(url.to_string()), Self::File)
    }
}

/// A range in a document, in LSP units (line, UTF-16 column), end exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    /// First position.
    pub start: LspPoint,
    /// Position after the last one.
    pub end: LspPoint,
}

impl Span {
    pub(crate) fn from_lsp(range: lsp_types::Range) -> Self {
        Self {
            start: point_from_lsp(range.start),
            end: point_from_lsp(range.end),
        }
    }

    pub(crate) fn to_lsp(self) -> lsp_types::Range {
        lsp_types::Range {
            start: point_to_lsp(self.start),
            end: point_to_lsp(self.end),
        }
    }
}

/// A span in a document.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location {
    /// The document.
    pub uri: DocUri,
    /// The range in it.
    pub span: Span,
}

impl Location {
    pub(crate) fn from_lsp(location: &lsp_types::Location) -> Self {
        Self {
            uri: DocUri::from_url(&location.uri),
            span: Span::from_lsp(location.range),
        }
    }
}

pub(crate) fn point_from_lsp(p: lsp_types::Position) -> LspPoint {
    LspPoint {
        line: p.line,
        character: p.character,
    }
}

pub(crate) fn point_to_lsp(p: LspPoint) -> lsp_types::Position {
    lsp_types::Position {
        line: p.line,
        character: p.character,
    }
}
