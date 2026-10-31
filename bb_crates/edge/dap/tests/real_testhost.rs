//! End-to-end VSTest host attach through the installed netcoredbg.
#![cfg(test)]
use std::path::PathBuf;
use std::time::{Duration, Instant};

use bb_dap::{DapSession, Event, Scope, SourceBreakpoint, StackFrame, Step, Variable};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::mpsc;

const DEADLINE: Duration = Duration::from_secs(30);
type Lines = tokio::io::Lines<BufReader<ChildStdout>>;

#[tokio::test]
async fn greeter_testhost_stops_reads_locals_steps_and_finishes() {
    if bb_dap::locate().is_err() {
        return;
    }
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let (mut child, _lines, process_id): (Child, Lines, u32) = start_testhost(&root).await;
    let source: PathBuf = root.join("fixtures/_Harness/_NUnitApex/GreeterSpec.cs");
    let mut session: DapSession = DapSession::spawn(DEADLINE).unwrap();
    let mut events: mpsc::Receiver<Event> = session.take_events().unwrap();
    configure(&session, process_id, &source).await;
    let (stops, saw_name, saw_expected): (usize, bool, bool) =
        consume_stops(&session, &mut events).await;
    assert!(stops >= 2);
    assert!(saw_name);
    assert!(saw_expected);
    let status: std::process::ExitStatus = tokio::time::timeout(DEADLINE, child.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(status.success());
}

async fn start_testhost(root: &std::path::Path) -> (Child, Lines, u32) {
    let project: PathBuf = root.join("fixtures/_Harness/_NUnitApex/_NUnitApex.csproj");
    let artifacts: PathBuf =
        std::env::temp_dir().join(format!("backback-testhost-{}", std::process::id()));
    let mut child: Child = Command::new("dotnet")
        .arg("test")
        .arg(&project)
        .arg("--artifacts-path")
        .arg(&artifacts)
        .args(["--filter", "FullyQualifiedName~GreeterSpec.Greets", "-v:q"])
        .env("VSTEST_HOST_DEBUG", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let stdout: ChildStdout = child.stdout.take().unwrap();
    let mut lines: Lines = BufReader::new(stdout).lines();
    let process_id: u32 = tokio::time::timeout(DEADLINE, async {
        loop {
            let line: String = lines.next_line().await.unwrap().unwrap();
            if let Some(rest) = line.strip_prefix("Process Id: ") {
                return rest.split(',').next().unwrap().parse::<u32>().unwrap();
            }
        }
    })
    .await
    .unwrap();
    (child, lines, process_id)
}

async fn configure(session: &DapSession, process_id: u32, source: &std::path::Path) {
    let caps: bb_dap::Capabilities = session.initialize().await.unwrap();
    assert!(!caps.supports_goto_targets_request);
    session.attach(process_id).await.unwrap();
    let breakpoints: Value = session
        .set_breakpoints(
            source,
            &[SourceBreakpoint {
                line: 13,
                condition: None,
            }],
        )
        .await
        .unwrap();
    assert_eq!(
        breakpoints
            .get("breakpoints")
            .and_then(Value::as_array)
            .and_then(|items: &Vec<Value>| items.first())
            .and_then(|item: &Value| item.get("line"))
            .and_then(Value::as_u64),
        Some(13)
    );
    session.configuration_done().await.unwrap();
}

async fn consume_stops(
    session: &DapSession,
    events: &mut mpsc::Receiver<Event>,
) -> (usize, bool, bool) {
    let started: Instant = Instant::now();
    let mut step_started: Option<Instant> = None;
    let mut stops: usize = 0;
    let mut saw_name: bool = false;
    let mut saw_expected: bool = false;
    loop {
        match DapSession::next_event(events, DEADLINE).await.unwrap() {
            Event::Stopped(body) => {
                stops += 1;
                if stops == 1 {
                    eprintln!("attach_to_first_stop_ms={}", started.elapsed().as_millis());
                }
                if let Some(at) = step_started.take() {
                    eprintln!("step_to_stop_ms={}", at.elapsed().as_millis());
                }
                let thread_id: i64 = body.get("threadId").and_then(Value::as_i64).unwrap();
                let (name, expected): (bool, bool) = read_locals(session, thread_id).await;
                saw_name |= name;
                saw_expected |= expected;
                if stops == 1 {
                    step_started = Some(Instant::now());
                    session.step(Step::Next, thread_id).await.unwrap();
                } else {
                    session.continue_thread(thread_id).await.unwrap();
                }
            }
            Event::Terminated(_) => break,
            Event::Output(_) | Event::Exited(_) | Event::Breakpoint(_) | Event::Other(_, _) => {}
        }
    }
    (stops, saw_name, saw_expected)
}

async fn read_locals(session: &DapSession, thread_id: i64) -> (bool, bool) {
    let frames: Vec<StackFrame> = session.stack_trace(thread_id).await.unwrap();
    let frame: &StackFrame = frames.first().unwrap();
    let scopes: Vec<Scope> = session.scopes(frame.id).await.unwrap();
    let mut saw_name: bool = false;
    let mut saw_expected: bool = false;
    for scope in scopes {
        let vars: Vec<Variable> = session.variables(scope.variables_reference).await.unwrap();
        saw_name |= vars.iter().any(|var: &Variable| var.name == "name");
        saw_expected |= vars.iter().any(|var: &Variable| var.name == "expected");
    }
    (saw_name, saw_expected)
}
