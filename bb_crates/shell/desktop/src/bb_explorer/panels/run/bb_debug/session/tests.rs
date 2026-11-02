//! The real netcoredbg on the `Debuggee` fixture: stop at a breakpoint with
//! the call stack and locals, step over, continue to the next hit, stop.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{start, Command, Frame, Note, Start, StepKind, Var};

/// `int result = n * n;` in `Square`, 0-based.
const BREAK: usize = 17;
/// `total += Square(i);` in `Main`, 0-based.
const CALL: usize = 9;

fn fixture() -> (tempfile::TempDir, PathBuf) {
    let source: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/Debuggee");
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    for name in ["Debuggee.csproj", "_Program.cs"] {
        std::fs::copy(source.join(name), dir.path().join(name)).unwrap();
    }
    let program: PathBuf = dir.path().join("_Program.cs").canonicalize().unwrap();
    (dir, program)
}

/// The next note that is not output, within `limit`.
fn next(notes: &mut UnboundedReceiver<Note>, limit: Duration) -> Note {
    let started: Instant = Instant::now();
    while started.elapsed() < limit {
        match notes.try_recv() {
            Ok(Note::Output(_)) => {}
            Ok(note) => return note,
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                panic!("the session closed its channel")
            }
        }
    }
    panic!("no note within {limit:?}");
}

fn stop(notes: &mut UnboundedReceiver<Note>, limit: Duration) -> (Vec<Frame>, Vec<Var>) {
    loop {
        match next(notes, limit) {
            Note::Stopped { frames, vars, .. } => return (frames, vars),
            Note::Running | Note::Building => {}
            other @ (Note::Variables { .. }
            | Note::Output(_)
            | Note::Ended(_)
            | Note::Failed(_)) => {
                panic!("expected a stop, got {other:?}")
            }
        }
    }
}

fn value<'a>(vars: &'a [Var], name: &str) -> Option<&'a str> {
    vars.iter()
        .find(|v: &&Var| v.name == name)
        .map(|v: &Var| v.value.as_str())
}

#[test]
fn a_real_session_stops_steps_continues_and_ends() {
    bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let (dir, program): (tempfile::TempDir, PathBuf) = fixture();
    let (commands, mut notes): (UnboundedSender<Command>, UnboundedReceiver<Note>) = start(Start {
        target: dir.path().join("Debuggee.csproj"),
        breakpoints: vec![(program.clone(), vec![BREAK])],
    });
    let (frames, vars): (Vec<Frame>, Vec<Var>) = stop(&mut notes, Duration::from_secs(240));
    let top: &Frame = frames.first().unwrap();
    assert!(top.name.contains("Square"), "{frames:?}");
    assert_eq!(
        (top.path.as_deref(), top.line),
        (Some(program.as_path()), BREAK)
    );
    let caller: &Frame = frames.get(1).unwrap();
    assert!(caller.name.contains("Main"), "{frames:?}");
    assert_eq!(caller.line, CALL, "the call site");
    assert_eq!(value(&vars, "n"), Some("1"), "{vars:?}");

    commands.send(Command::Step(StepKind::Over)).unwrap();
    let (frames, vars): (Vec<Frame>, Vec<Var>) = stop(&mut notes, Duration::from_secs(30));
    assert_eq!(frames.first().map(|f: &Frame| f.line), Some(BREAK + 1));
    assert_eq!(value(&vars, "result"), Some("1"), "{vars:?}");

    commands.send(Command::Continue).unwrap();
    let (_, vars): (Vec<Frame>, Vec<Var>) = stop(&mut notes, Duration::from_secs(30));
    assert_eq!(value(&vars, "n"), Some("2"), "the second hit");

    commands.send(Command::Stop).unwrap();
    assert_eq!(
        next(&mut notes, Duration::from_secs(30)),
        Note::Ended(String::from("stopped"))
    );
}
