//! An async spec under the real netcoredbg, as the headmate's (BBEOD25):
//! `async Task` with awaits, locals and a lambda that captures them. Set
//! Next moves the arrow up inside it (BB-BUG-071), and a local declared
//! after an await is listed and evaluates (BB-BUG-072's normal case).

use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;

use super::guard::Debugged;
use super::{next, spec_project, value};
use crate::run::bb_debug::session::{Command, Frame, Note, Start, StepKind, Var};

const ASYNC_SPEC: &str = "using System;
using System.Threading.Tasks;
using NUnit.Framework;
namespace Sample;
public class Spec
{
    [Test]
    public async Task Waits()
    {
        int count = 0;
        count += 1;
        await Task.Delay(20);
        int x = 1;
        Func<int, int> add = (int v) => v + count + x;
        count = add(count);
        await Task.Yield();
        Assert.That(count, Is.GreaterThan(0));
    }
}
";

/// The 0-based line of `statement` in [`ASYNC_SPEC`].
fn line(statement: &str) -> usize {
    ASYNC_SPEC
        .lines()
        .position(|l: &str| l.trim() == statement)
        .unwrap()
}

/// The next stop: its reason, top line, top frame id and variables.
fn stop(notes: &mut UnboundedReceiver<Note>, limit: Duration) -> (String, usize, i64, Vec<Var>) {
    loop {
        match next(notes, limit) {
            Note::Stopped {
                reason,
                frames,
                vars,
                ..
            } => {
                let top: &Frame = frames.first().unwrap();
                return (reason, top.line, top.id, vars);
            }
            Note::Running | Note::Building | Note::CanSetNext(_) => {}
            other @ (Note::Variables { .. }
            | Note::Children { .. }
            | Note::Evaluated { .. }
            | Note::TestResults { .. }
            | Note::Output(_)
            | Note::Refused(_)
            | Note::Ended(_)
            | Note::Failed(_)) => panic!("expected a stop, got {other:?}"),
        }
    }
}

#[test]
fn set_next_moves_up_inside_an_async_spec_and_its_late_local_shows() {
    bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    let spec: PathBuf = spec_project(&root, ASYNC_SPEC);
    let after: usize = line("count = add(count);");
    let mut run: Debugged = Debugged::start(Start {
        target: root.join("Tests.csproj"),
        test: Some((
            root.clone(),
            bb_testkit::Fqn::new("Sample.Spec.Waits").unwrap(),
        )),
        breakpoints: vec![(spec.clone(), vec![after])],
        exceptions: Vec::new(),
    });
    let (_, at, frame, vars): (String, usize, i64, Vec<Var>) =
        stop(&mut run.notes, Duration::from_secs(300));
    assert_eq!(at, after, "paused after the first await");
    assert_eq!(
        value(&vars, "x"),
        Some("1"),
        "a local after an await: {vars:?}"
    );
    assert_eq!(value(&vars, "count"), Some("1"), "{vars:?}");
    hover_value_is(&mut run, frame, "1");
    for (target, then) in [
        (
            "int x = 1;",
            "Func<int, int> add = (int v) => v + count + x;",
        ),
        ("count += 1;", "await Task.Delay(20);"),
    ] {
        set_next_then_step(&mut run, &spec, (target, then));
    }
    run.commands.send(Command::Stop).unwrap();
}

/// The hover's question for `x` in `frame`: the debugger answers `want`.
fn hover_value_is(run: &mut Debugged, frame: i64, want: &str) {
    run.commands
        .send(Command::Evaluate {
            expression: String::from("x"),
            frame,
            token: 7,
        })
        .unwrap();
    match next(&mut run.notes, Duration::from_secs(30)) {
        Note::Evaluated { token, value } => {
            assert_eq!(token, 7);
            assert_eq!(value.map(|v: Var| v.value), Ok(want.to_owned()));
        }
        other @ (Note::Building
        | Note::Running
        | Note::Stopped { .. }
        | Note::Variables { .. }
        | Note::Children { .. }
        | Note::TestResults { .. }
        | Note::Output(_)
        | Note::CanSetNext(_)
        | Note::Refused(_)
        | Note::Ended(_)
        | Note::Failed(_)) => panic!("expected the hover's value, got {other:?}"),
    }
}

/// Set Next up to `target`, then one step runs it and stops on `then`.
fn set_next_then_step(run: &mut Debugged, spec: &std::path::Path, (target, then): (&str, &str)) {
    run.commands
        .send(Command::SetNext(spec.to_path_buf(), line(target)))
        .unwrap();
    let (reason, at, _, _): (String, usize, i64, Vec<Var>) =
        stop(&mut run.notes, Duration::from_secs(30));
    assert_eq!(
        (reason.as_str(), at),
        ("goto", line(target)),
        "Set Next up to {target}"
    );
    run.commands.send(Command::Step(StepKind::Over)).unwrap();
    let (_, at, _, _): (String, usize, i64, Vec<Var>) =
        stop(&mut run.notes, Duration::from_secs(30));
    assert_eq!(at, line(then), "the moved line ran next");
}
