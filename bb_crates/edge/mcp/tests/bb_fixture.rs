//! MCP process and protocol checks against the local C# fixture only.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use bb_mcp::{McpError, Session};
use serde_json::json;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../shell/desktop/fixtures/_Harness/McpServer/_McpServer.csproj")
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry: std::fs::DirEntry = entry?;
        let name: std::ffi::OsString = entry.file_name();
        if name == "bin" || name == "obj" {
            continue;
        }
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &to.join(&name))?;
        } else {
            std::fs::copy(entry.path(), to.join(&name))?;
        }
    }
    Ok(())
}

fn fixture_with_build_warning() -> std::io::Result<PathBuf> {
    let root: PathBuf = std::env::temp_dir().join(format!("bb-mcp-warning-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&root);
    let source: PathBuf = fixture()
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .ok_or_else(|| std::io::Error::other("fixture project has no solution folder"))?;
    for project in ["McpServer", "Core"] {
        copy_tree(&source.join(project), &root.join(project))?;
    }
    std::fs::write(
        root.join("McpServer/Warn.cs"),
        "namespace Harness.McpServer;\npublic static class Warn { public static void W() { int unused; } }\n",
    )?;
    Ok(root)
}

fn group_alive(pid: u32) -> bool {
    Command::new("pgrep")
        .args(["-g", &pid.to_string()])
        .status()
        .is_ok_and(|status: std::process::ExitStatus| status.success())
}

#[tokio::test]
async fn greet_bad_arguments_and_stop_leave_no_group() {
    let mut session: Session = Session::start(&fixture()).await.unwrap();
    let pid: u32 = session.pid().unwrap();
    session.ping().await.unwrap();
    let tools: Vec<bb_mcp::Tool> = session.tools().await.unwrap();
    assert!(tools.iter().any(|tool: &bb_mcp::Tool| tool.name == "Greet"));
    let good: bb_mcp::CallResult = session.call("Greet", json!({"name":"Vinh"})).await.unwrap();
    assert!(!good.is_error);
    assert!(good
        .content
        .iter()
        .any(|item: &serde_json::Value| item.get("text") == Some(&json!("hi Vinh"))));
    let bad: bb_mcp::CallResult = session.call("Greet", json!({})).await.unwrap();
    assert!(bad.is_error);
    let mut restarted: Session = session.restart(&fixture()).await.unwrap();
    assert!(!group_alive(pid));
    restarted.ping().await.unwrap();
    let restarted_pid: u32 = restarted.pid().unwrap();
    let stopping: std::time::Instant = std::time::Instant::now();
    let status: std::process::ExitStatus = restarted.stop().await.unwrap();
    let stop_time: Duration = stopping.elapsed();
    assert!(status.success());
    assert!(stop_time < Duration::from_secs(1), "{stop_time:?}");
    assert!(!group_alive(restarted_pid));
}

#[tokio::test]
async fn killing_server_mid_call_fails_waiter_without_orphan() {
    let mut session: Session = Session::start(&fixture()).await.unwrap();
    let pid: u32 = session.pid().unwrap();
    let killer: tokio::task::JoinHandle<std::io::Result<std::process::ExitStatus>> =
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            tokio::process::Command::new("/bin/kill")
                .args(["-KILL", &format!("-{pid}")])
                .status()
                .await
        });
    let call: Result<bb_mcp::CallResult, McpError> = session.call("Hold", json!({})).await;
    assert!(matches!(
        call,
        Err(McpError::Exited { .. } | McpError::Io { .. })
    ));
    assert!(killer.await.unwrap().unwrap().success());
    assert!(!group_alive(pid));
}

#[tokio::test]
async fn startup_stderr_survives_the_handshake() {
    let mut session: Session = Session::start(&fixture()).await.unwrap();
    let logs: &mut tokio::sync::broadcast::Receiver<String> = session.logs();
    let line: String = tokio::time::timeout(Duration::from_secs(1), logs.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(line.contains("fixture boot"));
    session.stop().await.unwrap();
}

#[tokio::test]
async fn build_warnings_never_reach_the_protocol_channel() {
    let root: PathBuf = fixture_with_build_warning().unwrap();
    let started: Result<Session, McpError> =
        Session::start(&root.join("McpServer/_McpServer.csproj")).await;
    let mut session: Session = started.unwrap();
    session.ping().await.unwrap();
    session.stop().await.unwrap();
    std::fs::remove_dir_all(&root).unwrap();
}
