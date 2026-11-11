//! The real netcoredbg on the `Debuggee` fixture: stop at a breakpoint with
//! the call stack and locals, step over, continue to the next hit, stop;
//! the `all` exception filter pauses on the fixture's caught throw; Debug on
//! a test stops on a breakpoint inside it.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{start, Command, Frame, Note, Start, StepKind, Var};

/// `int result = n * n;` in `Square`, 0-based.
const BREAK: usize = 17;
/// `total += Square(i);` in `Main`, 0-based.
const CALL: usize = 9;
/// `throw new InvalidOperationException(...)` in `Fail`, 0-based.
const THROW: usize = 36;

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
            Note::Running | Note::Building | Note::CanSetNext(_) => {}
            other @ (Note::Variables { .. }
            | Note::Output(_)
            | Note::Ended(_)
            | Note::Failed(_)
            | Note::Refused(_)) => {
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
        test: None,
        breakpoints: vec![(program.clone(), vec![BREAK])],
        exceptions: Vec::new(),
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

/// The next stop with its reason and exception, within `limit`.
fn stop_with_reason(
    notes: &mut UnboundedReceiver<Note>,
    limit: Duration,
) -> (String, Option<String>, Vec<Frame>) {
    loop {
        match next(notes, limit) {
            Note::Stopped {
                reason,
                exception,
                frames,
                ..
            } => return (reason, exception, frames),
            Note::Running | Note::Building | Note::CanSetNext(_) => {}
            other @ (Note::Variables { .. }
            | Note::Output(_)
            | Note::Ended(_)
            | Note::Failed(_)
            | Note::Refused(_)) => {
                panic!("expected a stop, got {other:?}")
            }
        }
    }
}

/// BBEOD19: the `all` filter pauses on the Debuggee's caught throw, in
/// `Fail` at the throw line, and names the exception; with the filter
/// cleared, Continue runs the program to its end.
#[test]
fn the_all_filter_pauses_on_a_caught_throw_and_names_the_exception() {
    bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let (dir, program): (tempfile::TempDir, PathBuf) = fixture();
    let (commands, mut notes): (UnboundedSender<Command>, UnboundedReceiver<Note>) = start(Start {
        target: dir.path().join("Debuggee.csproj"),
        test: None,
        breakpoints: Vec::new(),
        exceptions: vec![String::from("all")],
    });
    let (reason, exception, frames): (String, Option<String>, Vec<Frame>) =
        stop_with_reason(&mut notes, Duration::from_secs(240));
    assert_eq!(reason, "exception");
    let what: String = exception.expect("the exception is named");
    assert!(
        what.starts_with("System.InvalidOperationException"),
        "{what}"
    );
    assert!(what.contains("total was 14"), "the message: {what}");
    let top: &Frame = frames.first().unwrap();
    assert!(top.name.contains("Fail"), "{frames:?}");
    assert_eq!(
        (top.path.as_deref(), top.line),
        (Some(program.as_path()), THROW)
    );

    commands.send(Command::Exceptions(Vec::new())).unwrap();
    commands.send(Command::Continue).unwrap();
    loop {
        match next(&mut notes, Duration::from_secs(30)) {
            Note::Ended(_) => break,
            Note::Running | Note::Building | Note::CanSetNext(_) => {}
            other @ (Note::Stopped { .. }
            | Note::Variables { .. }
            | Note::Output(_)
            | Note::Failed(_)
            | Note::Refused(_)) => panic!("expected the end, got {other:?}"),
        }
    }
}

const SPEC: &str = "using NUnit.Framework;\nnamespace Sample;\npublic class Spec\n{\n    [Test]\n    public void Hits()\n    {\n        int n = 41;\n        n += 1;\n        Assert.That(n, Is.EqualTo(42));\n    }\n}\n";
/// `n += 1;` in `Hits`, 0-based.
const IN_TEST: usize = 8;

/// Debug on a test (headmate 2026-11-11: "can't debug this test while set
/// the breakpoints"): the test starts paused, the debugger attaches, and
/// the test stops on the breakpoint inside it with its locals.
#[test]
fn debugging_a_test_stops_on_a_breakpoint_inside_it() {
    bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("Tests.csproj"), r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject></PropertyGroup><ItemGroup><PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.1" /><PackageReference Include="NUnit" Version="4.6.1" /><PackageReference Include="NUnit3TestAdapter" Version="5.0.0" /></ItemGroup></Project>"#).unwrap();
    let spec: PathBuf = root.join("Spec.cs");
    std::fs::write(&spec, SPEC).unwrap();
    let (commands, mut notes): (UnboundedSender<Command>, UnboundedReceiver<Note>) = start(Start {
        target: root.join("Tests.csproj"),
        test: Some((
            root.clone(),
            bb_testkit::Fqn::new("Sample.Spec.Hits").unwrap(),
        )),
        breakpoints: vec![(spec.clone(), vec![IN_TEST])],
        exceptions: Vec::new(),
    });
    let (frames, vars): (Vec<Frame>, Vec<Var>) = stop(&mut notes, Duration::from_secs(300));
    let top: &Frame = frames.first().unwrap();
    assert!(top.name.contains("Hits"), "{frames:?}");
    assert_eq!(
        (top.path.as_deref(), top.line),
        (Some(spec.as_path()), IN_TEST)
    );
    assert_eq!(value(&vars, "n"), Some("41"), "{vars:?}");
    commands.send(Command::Stop).unwrap();
}
