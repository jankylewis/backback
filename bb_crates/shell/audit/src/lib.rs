//! Convention audit for backback, and for any C#/Rust tree a user opens.
//!
//! Rules (docs/conventions): HD taxonomy (at most 5 visible entries per folder),
//! the underscore rule (C# trees only), size caps per file, function and type,
//! and explicit types (every Rust `let` and closure parameter, no C# `var`).

mod csharp;
mod lexer;
mod rust;
mod tree;

use std::path::{Path, PathBuf};

/// The caps, in one place. Change a number here and the whole gate follows.
pub mod caps {
    /// Visible entries per folder (HD taxonomy).
    pub const FOLDER_ENTRIES: usize = 5;
    /// Lines per Markdown file.
    pub const MD_LINES: usize = 100;
    /// Lines per Rust file, tests included.
    pub const RS_LINES: usize = 250;
    /// Lines per C# file.
    pub const CS_LINES: usize = 250;
    /// Lines per other text file (TOML, YAML, JSON, shell).
    pub const OTHER_LINES: usize = 200;
    /// Lines per function or method (Rust and C#).
    pub const FUNCTION_LINES: usize = 50;
    /// Lines per Rust `impl`/`struct`/`enum`/`trait` block.
    pub const RS_TYPE_LINES: usize = 200;
    /// Lines per C# class, struct, record or interface.
    pub const CS_CLASS_LINES: usize = 200;
    /// Parameters per function, `self` excluded.
    pub const FUNCTION_PARAMS: usize = 5;
}

/// Which rule a violation breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// More than [`caps::FOLDER_ENTRIES`] visible entries in a folder.
    FolderEntries,
    /// Underscore rule broken (C# trees: one `_` per folder; elsewhere none).
    Underscore,
    /// A file is over its line cap.
    FileLines,
    /// A function or method is over [`caps::FUNCTION_LINES`].
    FunctionLines,
    /// A type block is over its cap.
    TypeLines,
    /// A function has more than [`caps::FUNCTION_PARAMS`] parameters.
    FunctionParams,
    /// A Rust `let` without a type annotation.
    LetType,
    /// A Rust closure parameter without a type annotation.
    ClosureType,
    /// A C# `var` declaration.
    CsVar,
}

/// One broken rule at one place.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    /// File or folder, relative to the audit root.
    pub path: PathBuf,
    /// 1-based line, 0 for folder-level rules.
    pub line: usize,
    /// The rule.
    pub rule: Rule,
    /// What was found.
    pub detail: String,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} {:?} {}",
            self.path.display(),
            self.line,
            self.rule,
            self.detail
        )
    }
}

/// Audit errors.
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    /// A folder or file could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// The path.
        path: PathBuf,
        /// The cause.
        #[source]
        source: std::io::Error,
    },
}

/// Audit every folder and file under `root`. Sorted, deterministic output.
///
/// # Errors
/// Returns [`AuditError::Read`] when a folder or file cannot be read.
pub fn audit(root: &Path) -> Result<Vec<Violation>, AuditError> {
    let mut found: Vec<Violation> = tree::walk(root)?;
    found.sort();
    Ok(found)
}

/// A finding inside one source file, before the path is attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Finding {
    pub(crate) line: usize,
    pub(crate) rule: Rule,
    pub(crate) detail: String,
}

/// 1-based line number of byte offset `at` in `text`.
pub(crate) fn line_of(text: &str, at: usize) -> usize {
    let upto: &str = text.get(..at).unwrap_or(text);
    upto.matches('\n').count().saturating_add(1)
}

/// Index of the `}` matching the `{` at `open`, counted on stripped text.
pub(crate) fn matching_brace(text: &str, open: usize) -> Option<usize> {
    let mut depth: usize = 0;
    for (offset, ch) in text.get(open..)?.char_indices() {
        match ch {
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return open.checked_add(offset);
                }
            }
            _ => {}
        }
    }
    None
}
