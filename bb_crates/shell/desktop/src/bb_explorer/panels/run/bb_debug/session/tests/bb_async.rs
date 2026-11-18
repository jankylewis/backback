//! An async spec under the real netcoredbg, as the headmate's (BBEOD25):
//! `async Task` with awaits, locals and a lambda that captures them. Set
//! Next moves the arrow up inside it (BB-BUG-071), and a local declared
//! after an await is listed and evaluates (BB-BUG-072's normal case).
//! Variant-aware (BB-GAT): a stock netcoredbg (CI) has no
//! `gotoTargets`, so Set Next is the visible refusal and the arrow stays;
//! the patched local adapter must move it.

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

/// The adapter's own answer, asked apart from the session under test.
fn adapter_can_set_next() -> bool {
    let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut probe: bb_dap::DapSession =
            bb_dap::DapSession::spawn(Duration::from_secs(15)).unwrap();
        let supports: bool = probe
            .initialize()
            .await
            .unwrap()
            .supports_goto_targets_request;
        probe.stop_adapter();
        supports
    })
}

#[test]
fn set_next_moves_up_inside_an_async_spec_and_its_late_local_shows() {
    bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let patched: bool = adapter_can_set_next();
    eprintln!(
        "adapter_variant={}",
        if patched { "patched" } else { "stock" }
    );
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
    if patched {
        for (target, then) in [
            (
                "int x = 1;",
                "Func<int, int> add = (int v) => v + count + x;",
            ),
            ("count += 1;", "await Task.Delay(20);"),
        ] {
            set_next_then_step(&mut run, &spec, (target, then));
        }
    } else {
        set_next_is_refused_and_the_arrow_stays(&mut run, &spec);
    }
    run.commands.send(Command::Stop).unwrap();
}

/// A stock adapter: Set Next is the visible refusal, nothing moves, and
/// one step runs on from where the arrow was.
fn set_next_is_refused_and_the_arrow_stays(run: &mut Debugged, spec: &std::path::Path) {
    run.commands
        .send(Command::SetNext(spec.to_path_buf(), line("int x = 1;")))
        .unwrap();
    let refusal: String = format!(
        "Set Next Statement refused: {}",
        bb_dap::DapError::Unsupported("Set Next Statement")
    );
    assert_eq!(refused(&mut run.notes), refusal);
    run.commands.send(Command::Step(StepKind::Over)).unwrap();
    let (_, at, _, vars): (String, usize, i64, Vec<Var>) =
        stop(&mut run.notes, Duration::from_secs(30));
    assert_eq!(at, line("await Task.Yield();"), "the arrow did not move");
    assert_eq!(
        value(&vars, "x"),
        Some("1"),
        "the late local still shows: {vars:?}"
    );
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

/// The next refusal's text; a `CanSetNext` on the way is skipped.
fn refused(notes: &mut UnboundedReceiver<Note>) -> String {
    loop {
        match next(notes, Duration::from_secs(30)) {
            Note::Refused(text) => return text,
            Note::CanSetNext(_) => {}
            other @ (Note::Building
            | Note::Running
            | Note::Stopped { .. }
            | Note::Variables { .. }
            | Note::Children { .. }
            | Note::Evaluated { .. }
            | Note::TestResults { .. }
            | Note::Output(_)
            | Note::Ended(_)
            | Note::Failed(_)) => panic!("expected the visible refusal, got {other:?}"),
        }
    }
}
