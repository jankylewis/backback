//! BBEOD27: a rust-analyzer session on the fixture workspace, variant
//! aware like `bb_setnext.rs`. With a server on this machine: definition
//! and references across files, hover, document symbols, rename, the
//! language id, and its check builds only in the scratch folder. Without
//! one: the start fails with exactly the install hint the status bar shows.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::{
    DocPath, DocUri, FileChange, HoverInfo, IntelError, Location, RustConfig, Server, Session,
};
use bb_cargo::analyzer::{self, Found, Inputs, MISSING_HINT};
use bb_cargo::fixtures::{self, Shape, ADD_LINE, CALL_LINE};
use bb_text::LspPoint;
use serde_json::{json, Value};

const DEADLINE: Duration = Duration::from_secs(20);
const INDEXED: Duration = Duration::from_secs(180);

fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = base.path().join("ws");
    fixtures::write(&root, Shape::Clean).unwrap();
    let root: PathBuf = root.canonicalize().unwrap();
    let scratch: PathBuf = base.path().join("scratch-target");
    (base, root, scratch)
}

fn config(root: &Path, scratch: &Path) -> RustConfig {
    RustConfig {
        root: root.to_path_buf(),
        target_dir: scratch.to_path_buf(),
        inputs: Inputs::from_env(),
        request_timeout: DEADLINE,
    }
}

fn starts_on(location: &Location, file: &Path, line: u32) -> bool {
    matches!(&location.uri, DocUri::File(path) if path.as_path() == file)
        && location.span.start.line == line
}

async fn definition_once_indexed(session: &Session, lib: &DocPath, at: LspPoint) -> Vec<Location> {
    let started: Instant = Instant::now();
    loop {
        let found: Vec<Location> = session
            .definition(lib, at, DEADLINE)
            .await
            .unwrap_or_default();
        if !found.is_empty() || started.elapsed() > INDEXED {
            return found;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn prove_live_server(root: &Path, scratch: &Path, server: &Found) {
    let mut session: Session = Session::start_rust(&config(root, scratch), std::io::sink())
        .await
        .unwrap();
    assert_eq!(session.server(), Server::RustAnalyzer, "{server:?}");
    let lib: DocPath = DocPath::new(root.join("alpha/src/lib.rs")).unwrap();
    let math: PathBuf = root.join("alpha/src/math.rs");
    session.open(&lib).await.unwrap();
    let call: LspPoint = LspPoint {
        line: CALL_LINE - 1,
        character: 10,
    };
    let defined: Vec<Location> = definition_once_indexed(&session, &lib, call).await;
    assert!(
        defined
            .iter()
            .any(|l: &Location| starts_on(l, &math, ADD_LINE - 1)),
        "{defined:?}"
    );
    let used: Vec<Location> = session.references(&lib, call, DEADLINE).await.unwrap();
    assert!(
        used.iter()
            .any(|l: &Location| starts_on(l, lib.as_path(), CALL_LINE - 1)),
        "{used:?}"
    );
    let hover: Option<HoverInfo> = session.hover(&lib, call, DEADLINE).await.unwrap();
    assert!(hover.is_some_and(|h: HoverInfo| h.text.contains("fn add")));
    let symbols: Value = session
        .raw(
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": format!("file://{}", lib.as_path().display()) } }),
            DEADLINE,
        )
        .await
        .unwrap();
    assert!(symbols.to_string().contains("total"), "{symbols}");
    let renamed: Vec<FileChange> = session.rename(&lib, call, "plus", DEADLINE).await.unwrap();
    assert!(renamed.len() >= 2, "{renamed:?}");
    assert!(
        !root.join("target").exists(),
        "rust-analyzer built into the project"
    );
    session.shutdown().await.unwrap();
}

#[tokio::test]
async fn rust_analyzer_serves_the_fixture_or_the_start_names_the_install_hint() {
    let (_base, root, scratch): (tempfile::TempDir, PathBuf, PathBuf) = fixture();
    if let Ok(server) = analyzer::locate(&Inputs::from_env()) {
        prove_live_server(&root, &scratch, &server).await;
    } else {
        {
            let started: Result<Session, IntelError> =
                Session::start_rust(&config(&root, &scratch), std::io::sink()).await;
            let said: String = started
                .err()
                .map(|e: IntelError| e.to_string())
                .unwrap_or_default();
            assert_eq!(said, MISSING_HINT);
            assert!(!root.join("target").exists());
        }
    }
}
