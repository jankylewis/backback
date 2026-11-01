//! Stop is never queued behind slow actor work (B2, family "Stop waits
//! behind serial work"): it cancels a hung Start or an in-flight call and the
//! server is gone in < 300 ms.

use std::path::{Path, PathBuf};
use std::process::{Command as OsCommand, Stdio};
use std::time::{Duration, Instant};

use super::super::{Harness, ServerStatus};
use crate::harness::model::ToolView;

const STOP_BUDGET: Duration = Duration::from_millis(300);

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry: std::fs::DirEntry = entry.unwrap();
        let name: std::ffi::OsString = entry.file_name();
        if name == "bin" || name == "obj" {
            continue;
        }
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to.join(&name));
        } else {
            std::fs::copy(entry.path(), to.join(&name)).unwrap();
        }
    }
}

/// A fresh `_Harness` copy whose server never answers `initialize`.
fn hung_fixture() -> PathBuf {
    let made: PathBuf = std::env::temp_dir().join(format!("bb-mcp-hung-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&made);
    copy_tree(&super::render::fixture_root(), &made);
    let root: PathBuf = made.canonicalize().unwrap();
    let program: PathBuf = root.join("McpServer/Program.cs");
    let text: String = std::fs::read_to_string(&program).unwrap();
    let hung: String = text.replace(
        "await builder.Build().RunAsync();",
        "await System.Threading.Tasks.Task.Delay(-1);",
    );
    assert_ne!(text, hung);
    std::fs::write(&program, hung).unwrap();
    root
}

fn server_pids(root: &Path) -> Vec<u32> {
    let pattern: String = format!("{}/McpServer/bin/.*/McpServer.dll", root.display());
    let out: std::process::Output = OsCommand::new("pgrep")
        .args(["-f", &pattern])
        .stderr(Stdio::null())
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line: &str| line.trim().parse::<u32>().ok())
        .collect()
}

fn wait(deadline: Duration, harness: &mut Harness, done: impl Fn(&Harness) -> bool) -> bool {
    let started: Instant = Instant::now();
    while started.elapsed() < deadline {
        harness.poll();
        if done(harness) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

fn group_alive(group: u32) -> bool {
    OsCommand::new("pgrep")
        .args(["-g", &group.to_string()])
        .stdout(Stdio::null())
        .status()
        .unwrap()
        .success()
}

/// Stop, then time until the panel says Stopped and `left` is empty.
fn timed_stop(harness: &mut Harness, left: &dyn Fn() -> Vec<u32>) -> Duration {
    let started: Instant = Instant::now();
    harness.stop(0);
    let stopped: bool = wait(Duration::from_secs(40), harness, |state: &Harness| {
        state.servers[0].status == ServerStatus::Stopped && left().is_empty()
    });
    let took: Duration = started.elapsed();
    for pid in left() {
        // Clean up a red run so no server outlives the test.
        let _killed: std::io::Result<std::process::ExitStatus> = OsCommand::new("/bin/kill")
            .args(["-KILL", "--", &format!("-{pid}")])
            .status();
    }
    assert!(
        stopped,
        "Stop never finished: {:?}",
        harness.servers[0].status
    );
    took
}

#[test]
fn stop_cancels_a_hung_start_and_ends_the_server_within_300_ms() {
    let root: PathBuf = hung_fixture();
    let mut harness: Harness = Harness::load(&root).unwrap();
    harness.start(0);
    let started: Instant = Instant::now();
    while server_pids(&root).is_empty() {
        assert!(
            started.elapsed() < Duration::from_secs(120),
            "server never ran"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(harness.servers[0].status, ServerStatus::Starting);
    let took: Duration = timed_stop(&mut harness, &|| server_pids(&root));
    println!("stop during hung start: {took:?}");
    assert!(took < STOP_BUDGET, "stop during hung start took {took:?}");
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn stop_cancels_an_in_flight_call_and_ends_the_server_within_300_ms() {
    let root: PathBuf = super::render::fixture_root().canonicalize().unwrap();
    let mut harness: Harness = Harness::load(&root).unwrap();
    harness.selected = Some(0);
    harness.start(0);
    assert!(wait(
        Duration::from_secs(120),
        &mut harness,
        |state: &Harness| { state.servers[0].status == ServerStatus::Running }
    ));
    harness.tool = harness.servers[0]
        .tools
        .iter()
        .position(|tool: &ToolView| tool.name == "Hold");
    let group: u32 = harness.servers[0].pid.unwrap();
    harness.call();
    std::thread::sleep(Duration::from_millis(300));
    let left: &dyn Fn() -> Vec<u32> = &move || {
        if group_alive(group) {
            vec![group]
        } else {
            Vec::new()
        }
    };
    let took: Duration = timed_stop(&mut harness, left);
    println!("stop during in-flight call: {took:?}");
    assert!(
        took < STOP_BUDGET,
        "stop during in-flight call took {took:?}"
    );
}
