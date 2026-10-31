//! Code intelligence state of the window, kept apart from GPUI so every
//! rule here is tested without a window or a server.

mod bb_accept;
pub(crate) mod popups;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_intel::{Diagnostic, Severity};

#[cfg(test)]
pub(crate) use bb_accept::{fits, freshness, Freshness};
pub(crate) use bb_accept::{InFlight, Now, Pending, Question};
use popups::{ActionsPopup, CompletionPopup, HoverPopup, PlacesPanel, RenameInput, RenamePreview};

/// Where the solution load stands, as the status bar shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LoadStatus {
    /// The workspace holds no solution; intel stays off.
    NoSolution,
    /// Projects loaded so far.
    Loading { loaded: usize, total: usize },
    /// Every project loaded.
    Ready,
    /// The server failed; the text says why.
    Failed(String),
}

impl LoadStatus {
    /// The status bar text.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::NoSolution => String::from("No solution"),
            Self::Loading { loaded, total } => format!("Loading {loaded}/{total} projects"),
            Self::Ready => String::from("Ready"),
            Self::Failed(reason) => format!("C# server failed: {reason}"),
        }
    }
}

/// One answered question's stage times, for the latency budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TraceRow {
    /// In the queue before the server thread took it.
    pub(crate) queued: Duration,
    /// The server's own time.
    pub(crate) server: Duration,
    /// From the server's answer to the UI applying it.
    pub(crate) hop: Duration,
    /// UI frames between applying it and drawing it (0 = the next frame).
    pub(crate) frames: u64,
    /// From the server's answer to the frame that drew it.
    pub(crate) to_paint: Duration,
}

/// A cursor move waiting for its tab to finish opening.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Jump {
    /// The file.
    pub(crate) path: PathBuf,
    /// Where the cursor goes.
    pub(crate) at: bb_text::LspPoint,
    /// Open it read-only (outside the workspace, such as decompiled metadata).
    pub(crate) read_only: bool,
}

/// A completion answer applied but not yet drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Undrawn {
    pub(crate) stages: crate::intel::bb_worker::protocol::Stages,
    /// When the UI applied it.
    pub(crate) applied: std::time::Instant,
    /// The last frame drawn before it was applied.
    pub(crate) frame: u64,
}

/// Everything the window shows about code intelligence.
#[derive(Debug)]
pub(crate) struct IntelModel {
    /// The load.
    pub(crate) status: LoadStatus,
    /// Questions in flight.
    pub(crate) in_flight: InFlight,
    diagnostics: BTreeMap<PathBuf, Vec<Diagnostic>>,
    /// The completion list.
    pub(crate) completion: Option<CompletionPopup>,
    /// The hover text.
    pub(crate) hover: Option<HoverPopup>,
    /// The code actions list.
    pub(crate) actions: Option<ActionsPopup>,
    /// The references panel.
    pub(crate) references: Option<PlacesPanel>,
    /// The F2 box.
    pub(crate) rename_input: Option<RenameInput>,
    /// The rename plan awaiting Apply.
    pub(crate) rename_preview: Option<RenamePreview>,
    /// Whether the Problems panel shows.
    pub(crate) problems_open: bool,
    /// UI frames drawn so far.
    pub(crate) frame: u64,
    /// Completion latency rows, newest last.
    pub(crate) trace: Vec<TraceRow>,
    /// The completion answer waiting for its first frame.
    pub(crate) undrawn: Option<Undrawn>,
    /// A definition jump waiting for its tab.
    pub(crate) jump: Option<Jump>,
    /// The char the mouse rests on, with a token per move.
    pub(crate) hover_target: Option<(PathBuf, bb_text::CharIdx, u64)>,
    /// The new name of the rename in flight.
    pub(crate) rename_asked: Option<String>,
    /// Window position of the mouse when hover was asked.
    pub(crate) hover_at: (f32, f32),
}

impl IntelModel {
    pub(crate) fn new(status: LoadStatus) -> Self {
        Self {
            status,
            in_flight: InFlight::default(),
            diagnostics: BTreeMap::new(),
            completion: None,
            hover: None,
            actions: None,
            references: None,
            rename_input: None,
            rename_preview: None,
            problems_open: false,
            frame: 0,
            trace: Vec::new(),
            undrawn: None,
            jump: None,
            hover_target: None,
            rename_asked: None,
            hover_at: (0.0, 0.0),
        }
    }

    /// Keep the newest diagnostics of one document.
    pub(crate) fn set_diagnostics(&mut self, path: PathBuf, list: Vec<Diagnostic>) {
        self.diagnostics.insert(path, list);
    }

    /// The diagnostics of one document.
    pub(crate) fn diagnostics_of(&self, path: &Path) -> &[Diagnostic] {
        self.diagnostics.get(path).map_or(&[], Vec::as_slice)
    }

    /// Every diagnostic, by path, then line, then worst first.
    pub(crate) fn problems(&self) -> Vec<(&Path, &Diagnostic)> {
        let mut all: Vec<(&Path, &Diagnostic)> = self
            .diagnostics
            .iter()
            .flat_map(|(path, list): (&PathBuf, &Vec<Diagnostic>)| {
                list.iter().map(move |d: &Diagnostic| (path.as_path(), d))
            })
            .collect();
        all.sort_by_key(|(path, d): &(&Path, &Diagnostic)| {
            (
                *path,
                d.span.start.line,
                rank(d.severity),
                d.span.start.character,
            )
        });
        all
    }

    /// A tab closed: drop what belongs to it.
    pub(crate) fn closed(&mut self, path: &Path) {
        self.diagnostics.remove(path);
        self.in_flight.forget(path);
        if self
            .completion
            .as_ref()
            .is_some_and(|c: &CompletionPopup| c.path == path)
        {
            self.completion = None;
        }
        if self
            .rename_input
            .as_ref()
            .is_some_and(|r: &RenameInput| r.path == path)
        {
            self.rename_input = None;
        }
    }

    /// Close every popup (Escape, or a click elsewhere).
    pub(crate) fn dismiss(&mut self) -> bool {
        let open: bool = self.completion.is_some()
            || self.hover.is_some()
            || self.actions.is_some()
            || self.rename_input.is_some()
            || self.rename_preview.is_some();
        self.completion = None;
        self.hover = None;
        self.actions = None;
        self.rename_input = None;
        self.rename_preview = None;
        open
    }
}

/// Sort rank: errors first, unknown last.
pub(crate) fn rank(severity: Option<Severity>) -> u8 {
    match severity {
        Some(Severity::Error) => 0,
        Some(Severity::Warning) => 1,
        Some(Severity::Information) => 2,
        Some(Severity::Hint) => 3,
        None => 4,
    }
}
