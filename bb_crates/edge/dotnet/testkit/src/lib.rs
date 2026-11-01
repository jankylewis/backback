//! Headless test engine for backback.
//!
//! Two backends behind one trait, [`TestBackend`]:
//! - [`ApexBackend`]: the Apex CLI in IDE mode, as the Apex Testkit VS Code
//!   extension drives it (`<command> test --project P --list --json`, runs
//!   with `--specs`, `--trx`, `--debug`).
//! - [`DotnetBackend`]: plain `dotnet` for NUnit, xUnit and `MSTest` projects,
//!   on VSTest or Microsoft.Testing.Platform.
//!
//! Every process call has a deadline. A run is read from its TRX: a run
//! without TRX is an error, and a selected spec absent from TRX is an error
//! for that leaf ([`Leaf::Missing`]).

mod backend;
mod bb_engine;
mod model;
mod parse;

pub use backend::{ApexBackend, DebugHost, DotnetBackend, StopFlag, Workspace};
pub use bb_engine::{run, run_with_stop, Deadlines, Selection, TestBackend};
pub use model::{
    Category, Configuration, Fqn, FqnError, FqnParts, Leaf, Outcome, RunReport, SpecList,
    SpecResult, Verdict,
};
pub use parse::{
    parse_apex_list, parse_duration, parse_fqn_lines, parse_host_pid, parse_mtp_list, parse_trx,
    strip_jsonc, ApexList, ApexSettings, CommandLine, HostPid, OutputError, ParseFailure,
    SettingsError, TrxError, SETTINGS_FILE,
};

use std::path::PathBuf;
use std::time::Duration;

/// Everything the engine can fail with.
#[derive(Debug, thiserror::Error)]
pub enum TestkitError {
    /// The caller stopped the active process tree.
    #[error("test run stopped")]
    Cancelled,
    /// A process could not be started.
    #[error("cannot start {program}: {source}")]
    Spawn {
        /// The program.
        program: String,
        /// The cause.
        #[source]
        source: std::io::Error,
    },
    /// A process passed its deadline (it was killed).
    #[error("{what} timed out after {after:?}")]
    Timeout {
        /// What was running.
        what: String,
        /// The deadline.
        after: Duration,
    },
    /// A step that must succeed exited non-zero (list, build, target path).
    #[error("{what} exited with {code:?}: {detail}")]
    Failed {
        /// What was running.
        what: String,
        /// The exit code (`None`: killed by a signal).
        code: Option<i32>,
        /// The last lines of its error output.
        detail: String,
    },
    /// The run wrote no readable TRX, whatever its exit code.
    #[error("the run wrote no TRX at {0}")]
    NoTrx(PathBuf),
    /// The test assembly is not built (lists and runs use `--no-build`).
    #[error("not built: {0} (build the project first)")]
    NotBuilt(PathBuf),
    /// A debug run printed no test-host process id.
    #[error("the debug run printed no test host id within {0:?}")]
    NoHostPid(Duration),
    /// The backend cannot do what was asked.
    #[error("the {backend} backend cannot {what}")]
    Unsupported {
        /// The backend.
        backend: &'static str,
        /// What was asked.
        what: &'static str,
    },
    /// The TRX cannot be read.
    #[error(transparent)]
    Trx(#[from] TrxError),
    /// The list output cannot be read.
    #[error(transparent)]
    Output(#[from] OutputError),
    /// A listed name is not a test name.
    #[error(transparent)]
    Name(#[from] FqnError),
    /// The workspace settings cannot be used.
    #[error(transparent)]
    Settings(#[from] SettingsError),
    /// The .NET SDK cannot be found.
    #[error(transparent)]
    Sdk(#[from] bb_dotnet::SdkError),
    /// The project file cannot be read.
    #[error(transparent)]
    Project(#[from] bb_dotnet::ProjectError),
    /// A file or folder of the engine cannot be used.
    #[error("cannot use {path}: {source}")]
    Io {
        /// The path.
        path: PathBuf,
        /// The cause.
        #[source]
        source: std::io::Error,
    },
}
