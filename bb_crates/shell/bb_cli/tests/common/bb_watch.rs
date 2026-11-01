//! The run helper on a fake slow load: a shell that prints the readiness
//! line late, like `bb probe` under load (~42 s against a 30 s wall).

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::common::{watch, Pending, Plan, Run, Stalled};

fn shell(script: &str) -> Child {
    Command::new("/bin/sh")
        .args(["-c", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

/// A load longer than the after-ready budget, then a quick answer.
const SLOW_LOAD: &str =
    "echo 'log       : x'; sleep 2; echo 'loaded    : 2s'; echo 'result    : 1'";

fn roslyn(ready: u64, after: u64) -> Plan {
    Plan::UntilReady {
        ready: Duration::from_secs(ready),
        after: Duration::from_secs(after),
    }
}

#[test]
fn a_slow_load_waits_on_the_readiness_line_not_a_flat_wall() {
    let run: Result<Run, Stalled> = watch(shell(SLOW_LOAD), roslyn(10, 1));
    let run: Run = run.unwrap_or_else(|stalled: Stalled| panic!("{stalled}"));
    assert_eq!(run.code, Some(0));
    assert!(run.out.contains("result    : 1"), "{}", run.out);
}

#[test]
fn a_load_that_never_gets_ready_names_the_readiness_line_as_pending() {
    let stalled: Stalled = watch(shell("echo 'log       : x'; sleep 30"), roslyn(1, 1))
        .err()
        .unwrap();
    assert_eq!(stalled.pending, Pending::Ready);
    assert!(stalled.out.contains("log       : x"), "{stalled}");
    assert!(stalled.to_string().contains("loaded    :"), "{stalled}");
}

#[test]
fn a_request_that_hangs_after_ready_names_the_exit_as_pending() {
    let stalled: Stalled = watch(shell("echo 'loaded    : 0s'; sleep 30"), roslyn(5, 1))
        .err()
        .unwrap();
    assert_eq!(stalled.pending, Pending::Exit);
}

#[test]
fn a_plain_command_keeps_its_short_flat_deadline() {
    let stalled: Stalled = watch(shell("sleep 30"), Plan::Flat(Duration::from_secs(1)))
        .err()
        .unwrap();
    assert_eq!(stalled.pending, Pending::Exit);
}
