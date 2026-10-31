//! What the UI sends to the language server thread and what comes back.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bb_intel::{ActionItem, CompletionEntry, Diagnostic, Edit, FileChange, HoverInfo, Location};
use bb_text::LspPoint;

/// Which question an answer belongs to; only the newest one per kind counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Ticket(pub(crate) u64);

/// A question about one point (or range) of an open document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ask {
    /// Completion items at the point.
    Completion(LspPoint),
    /// Hover text at the point.
    Hover(LspPoint),
    /// Where the symbol at the point is defined.
    Definition(LspPoint),
    /// Every use of the symbol at the point.
    References(LspPoint),
    /// The edits that rename the symbol at the point.
    Rename(LspPoint, String),
    /// Code actions for a range.
    Actions(LspPoint, LspPoint),
}

/// The kind of an [`Ask`], one slot per kind in the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum AskKind {
    /// See [`Ask::Completion`].
    Completion,
    /// See [`Ask::Hover`].
    Hover,
    /// See [`Ask::Definition`].
    Definition,
    /// See [`Ask::References`].
    References,
    /// See [`Ask::Rename`].
    Rename,
    /// See [`Ask::Actions`].
    Actions,
}

impl Ask {
    pub(crate) fn kind(&self) -> AskKind {
        match self {
            Self::Completion(_) => AskKind::Completion,
            Self::Hover(_) => AskKind::Hover,
            Self::Definition(_) => AskKind::Definition,
            Self::References(_) => AskKind::References,
            Self::Rename(..) => AskKind::Rename,
            Self::Actions(..) => AskKind::Actions,
        }
    }
}

/// UI to server thread. Handled strictly in order, so an edit sent before a
/// question is always in the server's text when it answers.
#[derive(Clone, Debug)]
pub(crate) enum Request {
    /// `didOpen` with the buffer's text.
    Open { path: PathBuf, text: String },
    /// Incremental `didChange` from the buffer's char log.
    Change { path: PathBuf, edits: Vec<Edit> },
    /// `didClose`.
    Close { path: PathBuf },
    /// A question; the answer carries the ticket back.
    Ask {
        ticket: Ticket,
        path: PathBuf,
        ask: Ask,
        sent: Instant,
    },
}

/// A location with the text of its line, read on the server thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    /// Where.
    pub(crate) location: Location,
    /// The line it starts on, trimmed; empty when unreadable.
    pub(crate) preview: String,
}

/// The answer to an [`Ask`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Answer {
    /// Completion items.
    Completion(Vec<CompletionEntry>),
    /// Hover text, if any.
    Hover(Option<HoverInfo>),
    /// Definitions.
    Definition(Vec<Place>),
    /// References.
    References(Vec<Place>),
    /// The rename plan, never applied by the server thread.
    Rename(Vec<FileChange>),
    /// Code actions.
    Actions(Vec<ActionItem>),
    /// The request failed; the text says why.
    Failed(String),
}

/// When each stage of one question happened, for the latency budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Stages {
    /// The UI queued the request.
    pub(crate) sent: Instant,
    /// The server thread took it from the queue.
    pub(crate) started: Instant,
    /// The server answered.
    pub(crate) answered: Instant,
}

impl Stages {
    /// Time in the queue before the server thread took the request.
    pub(crate) fn queued(&self) -> Duration {
        self.started.saturating_duration_since(self.sent)
    }

    /// Time the server took to answer.
    pub(crate) fn server(&self) -> Duration {
        self.answered.saturating_duration_since(self.started)
    }
}

/// Server thread to UI.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Event {
    /// Projects loaded so far, of all in the solution.
    Progress { loaded: usize, total: usize },
    /// Every project has loaded.
    Ready { took: Duration },
    /// The server could not start or load; intel stays off.
    Failed(String),
    /// The diagnostics of one open document.
    Diagnostics {
        path: PathBuf,
        list: Vec<Diagnostic>,
    },
    /// The answer to the question with this ticket.
    Answer {
        ticket: Ticket,
        answer: Answer,
        stages: Stages,
    },
    /// A sync step failed; the document may need reopening.
    SyncFailed(String),
}
