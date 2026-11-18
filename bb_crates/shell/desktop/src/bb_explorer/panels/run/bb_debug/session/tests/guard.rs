//! A real session owned by its test (BB-PRC-008): the session thread holds
//! the paused `dotnet test` run, and a test binary that exits after a failed
//! assertion kills that thread without its drops, so the run was left
//! behind (orphaned, ppid 1). The guard stops the session and waits until
//! the thread has ended and reaped the run, on success and on a panic.

use std::path::Path;
use std::time::{Duration, Instant};

use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::run::bb_debug::session::{start, Command, Note, Start};

/// How long the drop waits for the session thread to end.
const END: Duration = Duration::from_secs(60);

/// A started session; dropping it stops it and waits for its thread.
pub(super) struct Debugged {
    pub(super) commands: UnboundedSender<Command>,
    pub(super) notes: UnboundedReceiver<Note>,
}

impl Debugged {
    pub(super) fn start(start_with: Start) -> Debugged {
        let (commands, notes): (UnboundedSender<Command>, UnboundedReceiver<Note>) =
            start(start_with);
        Debugged { commands, notes }
    }
}

impl Drop for Debugged {
    fn drop(&mut self) {
        // The session may have ended already: then nobody reads it.
        let _sent: Result<(), _> = self.commands.send(Command::Stop);
        let started: Instant = Instant::now();
        while started.elapsed() < END {
            match self.notes.try_recv() {
                Err(TryRecvError::Disconnected) => return,
                Err(TryRecvError::Empty) => std::thread::sleep(Duration::from_millis(50)),
                Ok(_) => {}
            }
        }
    }
}

/// Whether any process still names `root` on its command line.
fn running(root: &Path) -> bool {
    std::process::Command::new("pgrep")
        .arg("-f")
        .arg(root)
        .output()
        .is_ok_and(|out: std::process::Output| out.status.success())
}

/// BB-PRC-008: a dropped debug-test session leaves no `dotnet test` run,
/// as when a failed assertion unwinds the test.
#[test]
fn a_dropped_debug_test_session_leaves_no_test_run() {
    bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: std::path::PathBuf = dir.path().canonicalize().unwrap();
    let spec: std::path::PathBuf = super::spec_project(&root, super::SPEC);
    let mut run: Debugged = Debugged::start(Start {
        target: root.join("Tests.csproj"),
        test: Some((
            root.clone(),
            bb_testkit::Fqn::new("Sample.Spec.Hits").unwrap(),
        )),
        breakpoints: vec![(spec, vec![super::IN_TEST])],
        exceptions: Vec::new(),
    });
    let _paused: (Vec<super::Frame>, Vec<super::Var>) =
        super::stop(&mut run.notes, Duration::from_secs(300));
    assert!(running(&root), "the paused run holds the project");
    drop(run);
    assert!(!running(&root), "the run outlived its session");
}
