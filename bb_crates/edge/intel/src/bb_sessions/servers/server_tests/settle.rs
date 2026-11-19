//! BBEOD29: rust-analyzer answers `content modified` (-32801) to a request
//! that lands while it reloads its workspace. The request is asked again
//! within its own deadline, for every server, so a hover or F12 right after
//! the open never fails with that reply; a server that never settles still
//! ends at the deadline with the server's own error. The start has its own
//! deadline (BB-GAT-015: a short request deadline once bounded
//! `initialize` too, which a loaded CI runner passed).

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_python::pyright::Inputs;
use bb_text::LspPoint;

use crate::{DocPath, HoverInfo, IntelError, PythonConfig, Session};

/// The start's own deadline (spawn and `initialize`), never the short
/// deadline of the request under test.
const START: Duration = Duration::from_secs(60);
/// A start slower than the never-settles test's request deadline.
const SLOW_START: Duration = Duration::from_millis(2500);

const SERVER: &str = r#"#!/usr/bin/python3
import json, sys, time

def read():
    length = 0
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            sys.exit(0)
        if not line.strip():
            return json.loads(sys.stdin.buffer.read(length))
        name, _, value = line.partition(b":")
        if name.strip().lower() == b"content-length":
            length = int(value)

def send(message):
    body = json.dumps(message).encode()
    sys.stdout.buffer.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    sys.stdout.buffer.flush()

modified = 0
while True:
    message = read()
    method = message.get("method")
    if method == "exit":
        sys.exit(0)
    if "id" not in message or method is None:
        continue
    reply = {"jsonrpc": "2.0", "id": message["id"], "result": None}
    if method == "initialize":
        time.sleep(START_DELAY)
        reply["result"] = {"capabilities": {"hoverProvider": True}}
    elif method == "textDocument/hover" and modified < MODIFIED:
        modified += 1
        del reply["result"]
        reply["error"] = {"code": -32801, "message": "content modified"}
    elif method == "textDocument/hover":
        reply["result"] = {"contents": "def add(a: int, b: int) -> int"}
    send(reply)
"#;

fn executable(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A server that answers `initialize` after `start_delay` and `content
/// modified` to the first `modified` hovers.
fn settling_server(dir: &Path, modified: u32, start_delay: Duration) -> PathBuf {
    let server: PathBuf = dir.join("pyright-langserver");
    let text: String = SERVER.replacen(
        "modified = 0\n",
        &format!(
            "MODIFIED = {modified}\nSTART_DELAY = {}\nmodified = 0\n",
            start_delay.as_secs_f32()
        ),
        1,
    );
    executable(&server, &text);
    executable(&dir.join("pyright"), "#!/bin/sh\necho 'pyright 1.1.0'\n");
    server
}

async fn hover_on(
    (modified, start_delay): (u32, Duration),
    deadline: Duration,
) -> (Result<Option<HoverInfo>, IntelError>, Duration) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = base.path().canonicalize().unwrap();
    let file: PathBuf = root.join("cart.py");
    std::fs::write(&file, "total = add(1, 2)\n").unwrap();
    let config: PythonConfig = PythonConfig {
        root: root.clone(),
        interpreter: PathBuf::from("/usr/bin/python3"),
        inputs: Inputs {
            env: Some(settling_server(&root, modified, start_delay).into_os_string()),
            path: None,
        },
        request_timeout: START,
    };
    let mut session: Session = Session::start_python(&config, std::io::sink())
        .await
        .unwrap();
    let doc: DocPath = DocPath::new(file).unwrap();
    session.open(&doc).await.unwrap();
    let at: LspPoint = LspPoint {
        line: 0,
        character: 9,
    };
    let asked: Instant = Instant::now();
    let answer: Result<Option<HoverInfo>, IntelError> = session.hover(&doc, at, deadline).await;
    let took: Duration = asked.elapsed();
    session.shutdown().await.unwrap();
    (answer, took)
}

#[tokio::test]
async fn a_content_modified_reply_is_asked_again_and_the_hover_arrives() {
    let (answer, _took): (Result<Option<HoverInfo>, IntelError>, Duration) =
        hover_on((2, Duration::ZERO), Duration::from_secs(10)).await;
    let text: String = answer
        .unwrap()
        .map(|h: HoverInfo| h.text)
        .unwrap_or_default();
    assert!(text.contains("def add"), "{text}");
}

#[tokio::test]
async fn a_server_that_never_settles_ends_at_the_deadline_with_its_own_error() {
    let deadline: Duration = Duration::from_secs(2);
    let (answer, took): (Result<Option<HoverInfo>, IntelError>, Duration) =
        hover_on((u32::MAX, SLOW_START), deadline).await;
    let said: String = answer
        .err()
        .map(|e: IntelError| e.to_string())
        .unwrap_or_default();
    assert!(said.contains("content modified"), "{said}");
    assert!(took < deadline + Duration::from_secs(1), "{took:?}");
}
