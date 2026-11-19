//! One open file: its document, its syntax tree, and whether it may change.

use std::path::PathBuf;
use std::time::Instant;

use bb_syntax::{test_methods, LiveTree, Syntax, SyntaxError, TestMethod, Tree};
use bb_text::{Document, DocumentError, TextChange};

mod bb_window;
pub(crate) mod untitled;
use bb_window as window;

#[cfg(test)]
pub(crate) use bb_window::LARGE_FILE_BYTES;

pub(crate) struct Tab {
    pub(crate) path: PathBuf,
    pub(crate) document: Document,
    /// Highlight queries; parsing uses the parser inside `live`.
    pub(crate) highlighter: Syntax,
    /// Colours of an active language, or plain text (BBEOD26).
    pub(crate) mode: bb_lang::FileMode,
    pub(crate) live: LiveTree,
    pub(crate) test_methods: Vec<TestMethod>,
    /// Lines the tree covers; `None` for a file parsed whole.
    pub(crate) window_lines: Option<std::ops::Range<usize>>,
    /// No edit, save or rename may change it (decompiled metadata, a file
    /// the user cannot write).
    pub(crate) read_only: bool,
    /// Pixels the text has moved left (BB-BUG-013); the gutter stays.
    pub(crate) scroll_x: f32,
    /// The cursor the view last kept in sight; a move scrolls to the new one.
    pub(crate) sighted: Option<bb_text::CharIdx>,
    /// The widest line in sight, in pixels, plus some slack: the scroll's end.
    pub(crate) widest_px: f32,
    /// Pinned tabs stay left of the others and outlive the bulk closes
    /// that skip pinned tabs (BB-BUG-039).
    pub(crate) pinned: bool,
    /// The editor group it shows in (BB-BUG-040, BB-BUG-045).
    pub(crate) group: super::Group,
    /// Its screen rows while word wrap is on.
    pub(crate) wrap: Option<super::super::view::wrap::Wrapped>,
    /// The bracket pair at the cursor, kept per (revision, cursor).
    pub(crate) braces: std::cell::Cell<Option<BraceMemo>>,
}

type BraceMemo = (
    (u64, usize),
    Option<super::super::view::chrome::braces::Pair>,
);

/// A tab opened or closed; the code intelligence link reads these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TabEvent {
    /// The tab for this file opened.
    Opened(PathBuf),
    /// The tab for this file closed.
    Closed(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TabError {
    #[error("document: {0}")]
    Document(#[from] DocumentError),
    #[error("syntax: {0}")]
    Syntax(#[from] SyntaxError),
    #[error("file path: {0}")]
    Path(#[from] std::io::Error),
}

impl Tab {
    /// Read and parse a file shown as `mode` (the window's languages pick
    /// it). Slow on a big file: call it off the UI thread.
    pub(crate) fn load(path: PathBuf, mode: bb_lang::FileMode) -> Result<Self, TabError> {
        let started: Instant = Instant::now();
        let document: Document = Document::open(&path)?;
        let window_lines: Option<std::ops::Range<usize>> = window::first_window(&document);
        let mut syntax: Syntax = Syntax::for_lang(lang_of(mode))?;
        let tree: Tree = syntax.parse_rope_in(
            document.rope(),
            None,
            window::points(&document, window_lines.as_ref()),
        )?;
        let highlighter: Syntax = Syntax::for_lang(lang_of(mode))?;
        let read_only: bool = std::fs::metadata(&path)
            .is_ok_and(|meta: std::fs::Metadata| meta.permissions().readonly());
        eprintln!(
            "editor_open_ms={:.3}",
            started.elapsed().as_secs_f64() * 1000.0
        );
        let mut document: Document = document;
        if read_only {
            document.lock_writes();
        }
        let test_methods: Vec<TestMethod> = csharp_tests(mode, &tree, document.rope());
        Ok(Self {
            path,
            document,
            highlighter,
            mode,
            live: LiveTree::new(syntax, tree),
            test_methods,
            window_lines,
            read_only,
            scroll_x: 0.0,
            sighted: None,
            widest_px: 0.0,
            pinned: false,
            group: super::Group(0),
            wrap: None,
            braces: std::cell::Cell::new(None),
        })
    }

    /// Move the document's new edits onto the shown tree. O(depth) per edit;
    /// the parse itself runs later, off the UI thread.
    pub(crate) fn sync_edits(&mut self) {
        let changes: Vec<TextChange> = self.document.take_changes();
        self.live.edit(&changes);
    }

    /// The line Set Next Statement means by `line`: the first line of the
    /// statement that covers it, else `line` (BB-BUG-071).
    pub(crate) fn statement_line(&self, line: usize) -> usize {
        let indent: usize = self.document.line(line).map_or(0, |text: String| {
            text.len().saturating_sub(text.trim_start().len())
        });
        bb_syntax::statement_line(self.live.tree(), line, indent).unwrap_or(line)
    }

    /// Where Set Next Statement goes from the cursor: this file and the
    /// cursor line's statement.
    pub(crate) fn set_next_place(&self) -> (PathBuf, usize) {
        (
            self.path.clone(),
            self.statement_line(self.document.cursor_line()),
        )
    }

    pub(crate) fn refresh_test_methods(&mut self) {
        self.test_methods = csharp_tests(self.mode, self.live.tree(), self.document.rope());
    }

    /// Show the tab as `mode` (BB-PRF-009: the window's languages arrived
    /// after it opened): a new grammar and a fresh parse of its text. A tab
    /// whose background parse still runs keeps its mode (false); its parse
    /// end asks again.
    pub(crate) fn recolour(&mut self, mode: bb_lang::FileMode) -> Result<bool, TabError> {
        if self.mode == mode || self.live.parsing() {
            return Ok(self.mode == mode);
        }
        // The fresh parse below reads the whole current text.
        let _parsed_below: Vec<TextChange> = self.document.take_changes();
        let mut syntax: Syntax = Syntax::for_lang(lang_of(mode))?;
        let points: Option<(bb_text::BytePoint, bb_text::BytePoint)> =
            window::points(&self.document, self.window_lines.as_ref());
        let tree: Tree = syntax.parse_rope_in(self.document.rope(), None, points)?;
        self.highlighter = Syntax::for_lang(lang_of(mode))?;
        self.live = LiveTree::new(syntax, tree);
        self.mode = mode;
        self.braces.set(None);
        self.refresh_test_methods();
        Ok(true)
    }

    /// Make the tab read-only: no edit, and no save from any path.
    pub(crate) fn set_read_only(&mut self) {
        self.read_only = true;
        self.document.lock_writes();
    }

    /// Sync and parse on this thread, as a test oracle for the background path.
    #[cfg(test)]
    pub(crate) fn reparse(&mut self) -> Result<(), TabError> {
        self.sync_edits();
        self.live
            .parse_now(self.document.rope(), self.parse_window())?;
        self.refresh_test_methods();
        Ok(())
    }
}

/// A C# file's mode: the rename's loads (Roslyn renames C# only) and tests.
pub(crate) const CSHARP: bb_lang::FileMode = bb_lang::FileMode::Colours(bb_lang::Lang::CSharp);

/// The grammar of `mode`; None is plain text.
pub(crate) fn lang_of(mode: bb_lang::FileMode) -> Option<bb_lang::Lang> {
    match mode {
        bb_lang::FileMode::Colours(lang) => Some(lang),
        bb_lang::FileMode::Plain => None,
    }
}

/// A file's test methods: only a C#-coloured file has any, so Run / Debug
/// never appear in a Rust, Python or plain-text tab (BBEOD26).
fn csharp_tests(mode: bb_lang::FileMode, tree: &Tree, rope: &bb_text::Rope) -> Vec<TestMethod> {
    match mode {
        bb_lang::FileMode::Colours(bb_lang::Lang::CSharp) => test_methods(tree, rope),
        bb_lang::FileMode::Colours(bb_lang::Lang::Rust | bb_lang::Lang::Python)
        | bb_lang::FileMode::Plain => Vec::new(),
    }
}
