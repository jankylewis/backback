//! MCP process and protocol checks against the local C# fixture only.

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use bb_mcp::{McpError, Session};
use serde_json::json;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../shell/desktop/fixtures/_Harness/McpServer/_McpServer.csproj")
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
    let status: std::process::ExitStatus = restarted.stop().await.unwrap();
    assert!(status.success());
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
        Err(McpError::Exited { .. } | McpError::Io(_))
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
