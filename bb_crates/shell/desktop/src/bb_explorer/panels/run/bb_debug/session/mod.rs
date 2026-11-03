//! The debug session: a thread that owns netcoredbg, and how a stop is read.
//! The types here are the channel between the window and that thread.

mod bb_worker;
pub(crate) mod control;
mod stops;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

pub(crate) use bb_worker::start;

/// UI to session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// F5 while stopped.
    Continue,
    /// F10, F11, Shift+F11.
    Step(StepKind),
    /// Shift+F5: end the program and the session.
    Stop,
    /// A call-stack row was picked: its variables.
    Frame(i64),
    /// A file's breakpoints changed (0-based lines).
    Breakpoints(PathBuf, Vec<usize>),
    /// The exceptions to break on changed (the adapter's filter ids).
    Exceptions(Vec<String>),
}

/// A step command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StepKind {
    /// F10.
    Over,
    /// F11.
    Into,
    /// Shift+F11.
    Out,
}

/// One frame of a stop. `path` is `None` for code with no source (the
/// framework); such frames are listed but cannot be opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Frame {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) path: Option<PathBuf>,
    /// 0-based.
    pub(crate) line: usize,
}

/// One variable: name and value as the adapter shows them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Var {
    pub(crate) name: String,
    pub(crate) value: String,
}

/// Session to UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Note {
    /// Building the target.
    Building,
    /// The program runs.
    Running,
    /// Stopped: frames top first, and the top frame's variables. A stop on
    /// an exception names it, `Type: message`.
    Stopped {
        reason: String,
        exception: Option<String>,
        frames: Vec<Frame>,
        vars: Vec<Var>,
    },
    /// The variables of a picked frame.
    Variables { frame: i64, vars: Vec<Var> },
    /// Program or debugger output.
    Output(String),
    /// The session ended (program exit or Stop).
    Ended(String),
    /// The session could not start or broke.
    Failed(String),
}

/// What to debug: a `.csproj` (built first) or a `.dll`, every file's
/// breakpoints (0-based lines) and the exceptions to break on (the
/// adapter's filter ids).
pub(crate) struct Start {
    pub(crate) target: PathBuf,
    pub(crate) breakpoints: Vec<(PathBuf, Vec<usize>)>,
    pub(crate) exceptions: Vec<String>,
}
