//! BBEOD28: a pyright session on the Python fixture, variant aware like
//! the rust-analyzer one. With pyright on this machine: definition and
//! references across modules, hover, completion (BBEOD29), document
//! symbols, rename, pushed diagnostics on the syntax error, the language
//! id, and no bytecode in the project. Without one: the start fails with
//! exactly the install hint, unless `BACKBACK_REQUIRE_TOOLS` is set.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::{
    CompletionEntry, DocPath, DocUri, FileChange, HoverInfo, IntelError, Location, PythonConfig,
    Server, Session,
};
use bb_python::fixtures::{self, Shape};
use bb_python::pyright::{self, Inputs, MISSING_HINT};
use bb_text::LspPoint;
use serde_json::{json, Value};

const DEADLINE: Duration = Duration::from_secs(20);
/// The start's own deadline (spawn and `initialize`), apart from the
/// requests' [`DEADLINE`] (BB-GAT-015).
const START: Duration = Duration::from_secs(60);
const INDEXED: Duration = Duration::from_secs(120);

fn fixture() -> (tempfile::TempDir, PathBuf) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = base.path().join("py");
    fixtures::write(&root, Shape::Broken).unwrap();
    let root: PathBuf = root.canonicalize().unwrap();
    (base, root)
}

fn config(root: &Path) -> PythonConfig {
    PythonConfig {
        root: root.to_path_buf(),
        interpreter: PathBuf::from("/usr/bin/python3"),
        inputs: Inputs::from_env(),
        request_timeout: START,
    }
}

fn starts_on(location: &Location, file: &Path, line: u32) -> bool {
    matches!(&location.uri, DocUri::File(path) if path.as_path() == file)
        && location.span.start.line == line
}

fn no_bytecode(root: &Path) -> bool {
    !root.join("shop/__pycache__").exists() && !root.join("tests/__pycache__").exists()
}

async fn definition_once_indexed(session: &Session, cart: &DocPath, at: LspPoint) -> Vec<Location> {
    let started: Instant = Instant::now();
    loop {
        let found: Vec<Location> = session
            .definition(cart, at, DEADLINE)
            .await
            .unwrap_or_default();
        if !found.is_empty() || started.elapsed() > INDEXED {
            return found;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn pushed_errors(session: &Session, broken: &DocPath) -> Vec<u32> {
    let started: Instant = Instant::now();
    loop {
        let lines: Vec<u32> = session
            .diagnostics(broken, DEADLINE)
            .await
            .map(|r: crate::DiagnosticsReport| {
                r.list
                    .iter()
                    .map(|d: &crate::Diagnostic| d.span.start.line)
                    .collect()
            })
            .unwrap_or_default();
        if !lines.is_empty() || started.elapsed() > INDEXED {
            return lines;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn prove_completion(session: &Session, cart: &DocPath) {
    let after_add: LspPoint = LspPoint {
        line: fixtures::CALL_LINE - 1,
        character: fixtures::CALL_COLUMN + 3,
    };
    let offered: Vec<CompletionEntry> =
        session.completion(cart, after_add, DEADLINE).await.unwrap();
    assert!(
        offered.iter().any(|c: &CompletionEntry| c.label == "add"),
        "{offered:?}"
    );
}

async fn prove_live_server(root: &Path) {
    let mut session: Session = Session::start_python(&config(root), std::io::sink())
        .await
        .unwrap();
    assert_eq!(session.server(), Server::Pyright);
    let cart: DocPath = DocPath::new(root.join(fixtures::CART_FILE)).unwrap();
    let math: PathBuf = root.join(fixtures::MATH_FILE);
    session.open(&cart).await.unwrap();
    let call: LspPoint = LspPoint {
        line: fixtures::CALL_LINE - 1,
        character: fixtures::CALL_COLUMN,
    };
    let defined: Vec<Location> = definition_once_indexed(&session, &cart, call).await;
    assert!(
        defined
            .iter()
            .any(|l: &Location| starts_on(l, &math, fixtures::ADD_LINE - 1)),
        "{defined:?}"
    );
    let used: Vec<Location> = session.references(&cart, call, DEADLINE).await.unwrap();
    assert!(
        used.iter()
            .any(|l: &Location| starts_on(l, cart.as_path(), fixtures::CALL_LINE - 1)),
        "{used:?}"
    );
    let hover: Option<HoverInfo> = session.hover(&cart, call, DEADLINE).await.unwrap();
    assert!(hover.is_some_and(|h: HoverInfo| h.text.contains("add")));
    let symbols: Value = session
        .raw(
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": format!("file://{}", cart.as_path().display()) } }),
            DEADLINE,
        )
        .await
        .unwrap();
    assert!(symbols.to_string().contains("total"), "{symbols}");
    let renamed: Vec<FileChange> = session.rename(&cart, call, "plus", DEADLINE).await.unwrap();
    assert!(renamed.len() >= 2, "{renamed:?}");
    prove_completion(&session, &cart).await;
    let broken: DocPath = DocPath::new(root.join(fixtures::BROKEN_FILE)).unwrap();
    session.open(&broken).await.unwrap();
    assert!(pushed_errors(&session, &broken)
        .await
        .contains(&(fixtures::BROKEN_LINE - 1)));
    assert!(no_bytecode(root), "pyright wrote bytecode into the project");
    session.shutdown().await.unwrap();
}

#[tokio::test]
async fn pyright_serves_the_fixture_or_the_start_names_the_install_hint() {
    let (_base, root): (tempfile::TempDir, PathBuf) = fixture();
    if pyright::locate(&Inputs::from_env()).is_ok() {
        prove_live_server(&root).await;
    } else {
        assert!(
            std::env::var_os("BACKBACK_REQUIRE_TOOLS").is_none(),
            "BACKBACK_REQUIRE_TOOLS is set but no pyright runs"
        );
        let started: Result<Session, IntelError> =
            Session::start_python(&config(&root), std::io::sink()).await;
        let said: String = started
            .err()
            .map(|e: IntelError| e.to_string())
            .unwrap_or_default();
        assert_eq!(said, MISSING_HINT);
        assert!(no_bytecode(&root));
    }
}

#[test]
fn a_python_document_is_sent_as_python_and_its_diagnostics_are_pushed() {
    assert_eq!(Server::Pyright.language_id(), "python");
    assert!(Server::Pyright.pushes_diagnostics());
    assert_eq!(Server::Pyright.name(), "Python server");
}
