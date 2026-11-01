//! Where the solution load stands, as the status bar shows it.

/// Where the solution load stands, as the status bar shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LoadStatus {
    /// The workspace holds no solution; intel stays off.
    NoSolution,
    /// Projects loaded so far.
    Loading { loaded: usize, total: usize },
    /// Every project loaded.
    Ready,
    /// The active server connection closed.
    Stopped,
    /// The server failed; the text says why.
    Failed(String),
}

impl LoadStatus {
    /// A batch line counted `loaded` of `total` projects. Only a load in
    /// progress moves: Roslyn logs batch lines after the load too (a
    /// decompiled metadata document loads one), and they must not turn
    /// `Ready` or `Failed` back into `Loading`.
    pub(crate) fn progress(&mut self, loaded: usize, total: usize) {
        match self {
            Self::Loading { .. } => *self = Self::Loading { loaded, total },
            Self::NoSolution | Self::Ready | Self::Stopped | Self::Failed(_) => {}
        }
    }

    /// The status bar text.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::NoSolution => String::from("No solution"),
            Self::Loading { loaded, total } => format!("Loading {loaded}/{total} projects"),
            Self::Ready => String::from("Ready"),
            Self::Stopped => String::from("C# server stopped — Restart"),
            Self::Failed(reason) => format!("C# server failed: {reason}"),
        }
    }
}
