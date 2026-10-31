//! Live DAP instrument test for Set Next Statement.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]

#[path = "setnext/bb_session.rs"]
mod bb_session;

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_dap::{DapError, DapSession};

use bb_session::Session;
use serde_json::{json, Value};
use tokio::process::Command;

fn fixture_line(source: &str, statement: &str) -> usize {
    source
        .lines()
        .position(|line: &str| line.trim() == statement)
        .unwrap()
        + 1
}

async fn build_fixture() -> (PathBuf, PathBuf, String) {
    let fixture: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture");
    let scratch: PathBuf =
        std::env::temp_dir().join(format!("backback-setnext-{}", std::process::id()));
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).unwrap();
    }
    std::fs::create_dir_all(&scratch).unwrap();
    let source: PathBuf = scratch.join("_Program.cs");
    std::fs::copy(fixture.join("_Program.cs"), &source).unwrap();
    std::fs::copy(
        fixture.join("GotoFixture.csproj"),
        scratch.join("GotoFixture.csproj"),
    )
    .unwrap();
    let output: std::process::Output = tokio::time::timeout(
        Duration::from_secs(60),
        Command::new("dotnet")
            .arg("build")
            .arg(scratch.join("GotoFixture.csproj"))
            .arg("-c")
            .arg("Debug")
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        output.status.success(),
        "fixture build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let content: String = std::fs::read_to_string(&source).unwrap();
    (scratch, source.canonicalize().unwrap(), content)
}

async fn first_stop(session: &mut Session, source: &Path, content: &str) -> i64 {
    let line: usize = fixture_line(content, "Console.WriteLine(count);");
    let source_path: String = source.to_string_lossy().into_owned();
    let response: (bool, Value) = session
        .request("initialize", json!({"adapterID":"backback"}))
        .await;
    assert!(response.0 && response.1["supportsGotoTargetsRequest"] == true);
    let assembly: PathBuf = source
        .parent()
        .unwrap()
        .join("bin/Debug/net10.0/GotoFixture.dll");
    assert!(
        session
            .request(
                "launch",
                json!({"program":assembly,"cwd":source.parent(),"stopAtEntry":true})
            )
            .await
            .0
    );
    assert!(session.request("configurationDone", json!({})).await.0);
    let entry: Value = session.event("stopped", Some("entry")).await;
    let thread: i64 = entry["threadId"].as_i64().unwrap();
    let catch_line: usize = fixture_line(content, "Console.WriteLine(++count);");
    let breakpoints: Value = json!({"source":{"path":source_path},"breakpoints":
        [{"line":line},{"line":catch_line},{"line":catch_line+4}]});
    assert!(session.request("setBreakpoints", breakpoints).await.0);
    assert!(
        session
            .request("continue", json!({"threadId":thread}))
            .await
            .0
    );
    session.event("stopped", Some("breakpoint")).await;
    thread
}

async fn rewind_and_reexecute(session: &mut Session, source: &Path, content: &str, thread: i64) {
    let source_path: String = source.to_string_lossy().into_owned();
    let other: usize = fixture_line(content, "Console.WriteLine(\"other\");");
    let cross: (bool, Value) = session
        .request(
            "gotoTargets",
            json!({"source":{"path":source_path},"line":other}),
        )
        .await;
    assert!(cross.0 && cross.1["targets"].as_array().unwrap().is_empty());
    let rewind: usize = fixture_line(content, "count += 1;");
    let targets: (bool, Value) = session
        .request(
            "gotoTargets",
            json!({"source":{"path":source_path},"line":rewind}),
        )
        .await;
    assert!(targets.0);
    let target: i64 = targets.1["targets"][0]["id"].as_i64().unwrap();
    assert!(
        session
            .request("goto", json!({"threadId":thread,"targetId":target}))
            .await
            .0
    );
    session.event("stopped", Some("goto")).await;
    assert!(
        !session
            .request("goto", json!({"threadId":thread,"targetId":target}))
            .await
            .0
    );
    for expected in ["count = count + 1;", "Console.WriteLine(count);"] {
        assert!(session.request("next", json!({"threadId":thread})).await.0);
        session.event("stopped", Some("step")).await;
        let trace: (bool, Value) = session
            .request("stackTrace", json!({"threadId":thread}))
            .await;
        assert_eq!(
            trace.1["stackFrames"][0]["line"],
            fixture_line(content, expected)
        );
    }
    observed_count_is_four(session, thread).await;
}

async fn observed_count_is_four(session: &mut Session, thread: i64) {
    let trace: (bool, Value) = session
        .request("stackTrace", json!({"threadId":thread}))
        .await;
    let frame: Value = trace.1["stackFrames"][0]["id"].clone();
    let value: (bool, Value) = session
        .request("evaluate", json!({"expression":"count","frameId":frame}))
        .await;
    assert!(value.0);
    assert_eq!(value.1["result"], "4");
}

async fn exception_exits_fail(session: &mut Session, source: &Path, content: &str, thread: i64) {
    let after: usize = fixture_line(content, "return 0;") - 1;
    let source_path: String = source.to_string_lossy().into_owned();
    for _region in ["catch", "finally"] {
        assert!(
            session
                .request("continue", json!({"threadId":thread}))
                .await
                .0
        );
        session.event("stopped", Some("breakpoint")).await;
        let targets: (bool, Value) = session
            .request(
                "gotoTargets",
                json!({"source":{"path":source_path},"line":after}),
            )
            .await;
        assert!(targets.0 && targets.1["targets"].as_array().unwrap().is_empty());
    }
    assert!(
        session
            .request("continue", json!({"threadId":thread}))
            .await
            .0
    );
    let exited: Value = session.event("exited", None).await;
    assert_eq!(exited["exitCode"], 0);
}

#[tokio::test]
async fn setnext_reexecutes_and_refuses_unsafe_targets() {
    let binary: PathBuf = bb_dap::locate().expect("netcoredbg must be installed for this gate");
    let capability_session: DapSession = DapSession::spawn(Duration::from_secs(15)).unwrap();
    let capabilities: bb_dap::Capabilities = capability_session.initialize().await.unwrap();
    assert_eq!(
        capability_session.can_set_next_statement(),
        capabilities.supports_goto_targets_request
    );
    if !capabilities.supports_goto_targets_request {
        assert!(matches!(
            capability_session.goto(1, 1).await,
            Err(DapError::Unsupported("Set Next Statement"))
        ));
        eprintln!("adapter_variant=stock; Set Next hidden; goto not sent");
        return;
    }
    eprintln!("adapter_variant=patched; goto readback required");
    let binary: String = binary.to_string_lossy().into_owned();
    let (scratch, source, content): (PathBuf, PathBuf, String) = build_fixture().await;
    let mut session: Session = Session::start(&binary);
    let thread: i64 = first_stop(&mut session, &source, &content).await;
    rewind_and_reexecute(&mut session, &source, &content, thread).await;
    exception_exits_fail(&mut session, &source, &content, thread).await;
    drop(session);
    std::fs::remove_dir_all(scratch).unwrap();
}
