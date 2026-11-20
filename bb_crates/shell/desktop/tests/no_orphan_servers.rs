//! BBEOD42 (BB-PRC-009): a real `backback` on a C# + Rust + Python folder
//! ends in every way a user or a test run can end it; no process it
//! started (Roslyn and its `BuildHost` and MSBuild node, rust-analyzer,
//! pyright) is left after a few seconds. And a launch ends only what a dead
//! backback left, never another program's process.

#[cfg(test)]
#[path = "orphans/bb_tree.rs"]
mod bb_tree;
#[cfg(test)]
#[path = "orphans/hang.rs"]
mod hang;
#[cfg(test)]
#[path = "orphans/run.rs"]
mod run;
#[cfg(test)]
#[path = "orphans/stand_in.rs"]
mod stand_in;

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use bb_tree::{launch, App};
use run::{building, building_on, nothing_left, running, running_with, Running};

#[test]
fn a_sigkilled_backback_leaves_no_process_behind() {
    let mut run: Running = running("wait:ready mark:up sleep:600000", false, "bb_mark up");
    run.app.child.kill().unwrap();
    nothing_left(run, "SIGKILL");
}

#[test]
fn a_sigkilled_backback_while_debugging_leaves_no_process_behind() {
    let mut run: Running = running_with(
        "wait:ready f5 mark:debugging sleep:600000",
        false,
        "bb_mark debugging",
        &["netcoredbg", "bin/Debug/net10.0/App"],
    );
    run.app.child.kill().unwrap();
    nothing_left(run, "SIGKILL while debugging");
}

#[test]
fn a_sigkilled_backback_while_it_builds_leaves_no_process_behind() {
    let mut run: Running = building("cmd-shift-b mark:building sleep:600000", "bb_mark building");
    run.app.child.kill().unwrap();
    nothing_left(run, "SIGKILL while it builds");
}

#[test]
fn a_sigkilled_backback_while_the_debugger_builds_leaves_no_process_behind() {
    let mut run: Running = building(
        "wait:ready f5 mark:debugbuild sleep:600000",
        "bb_mark debugbuild",
    );
    run.app.child.kill().unwrap();
    nothing_left(run, "SIGKILL while the debugger builds");
}

#[test]
fn a_sigtermed_backback_leaves_no_process_behind() {
    let run: Running = running("wait:ready mark:up sleep:600000", false, "bb_mark up");
    let _sent: std::process::ExitStatus = Command::new("/bin/kill")
        .args(["-TERM", &run.pid.to_string()])
        .status()
        .unwrap();
    nothing_left(run, "SIGTERM");
}

#[test]
fn a_backback_that_quits_leaves_no_process_behind() {
    let mut run: Running = running("wait:ready mark:up wait:go", true, "bb_mark up");
    run.app.go();
    assert!(run.app.ended_within(Duration::from_secs(30)), "quit");
    nothing_left(run, "quit");
}

#[test]
fn a_backback_that_panics_while_it_builds_leaves_no_process_behind() {
    let mut run: Running = building_on(
        "cmd-shift-b mark:building wait:go crash:panic",
        "bb_mark building",
        true,
    );
    run.app.go();
    assert!(
        run.app.ended_within(Duration::from_secs(30)),
        "the panic ends it"
    );
    assert!(run.app.printed("bb_crash panic", Duration::ZERO));
    nothing_left(run, "panic while it builds");
}

#[test]
fn a_force_quit_at_the_unsaved_tabs_prompt_while_it_builds_leaves_no_process_behind() {
    let mut run: Running = building(
        "cmd-shift-b x cmd-q mark:prompt sleep:600000",
        "bb_mark prompt",
    );
    assert!(
        run.app.child.try_wait().unwrap().is_none(),
        "the unsaved tab holds the quit at its prompt"
    );
    run.app.child.kill().unwrap();
    nothing_left(run, "unsaved tabs + force quit while it builds");
}

#[test]
fn a_launch_ends_only_the_groups_a_dead_backback_left() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let empty: PathBuf = dir.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let state: PathBuf = dir.path().join("state");
    let left: u32 = stand_in::group();
    let reused: u32 = stand_in::group();
    let stranger: u32 = stand_in::group();
    stand_in::dead_record(
        &state.join("children"),
        &[
            (left, stand_in::token(left)),
            (reused, String::from(stand_in::NEVER)),
        ],
    );
    let mut app: App = launch(&empty, &state, "", false);
    let printed: bool = app.printed("bb_orphans", Duration::from_secs(60));
    let ended: bool = stand_in::gone_within(left, Duration::from_secs(5));
    let kept: (bool, bool) = (stand_in::alive(reused), stand_in::alive(stranger));
    app.child.kill().unwrap();
    let _reaped: std::io::Result<std::process::ExitStatus> = app.child.wait();
    stand_in::kill(&[left, reused, stranger]);
    assert!(printed, "said {:?}", app.said.lock().unwrap());
    assert!(ended, "the dead backback's group ended");
    assert_eq!(kept, (true, true), "a reused pid and a stranger are kept");
}
