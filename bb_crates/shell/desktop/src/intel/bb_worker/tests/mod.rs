//! The server thread against the real roslyn-language-server on a copy of
//! the harness: load progress, sync, answers, stage times.

#[path = "bb_death.rs"]
mod death;
mod events;
mod trace;

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_intel::{Diagnostic, DocUri, Edit, FileChange, Severity};
use bb_text::{Buffer, CharIdx, LspPoint};
use tokio::sync::mpsc::UnboundedReceiver;

use super::protocol::{Answer, Ask, Event, Place, Request, Stages, Ticket};
use super::{find_solution, start, Link};
use events::{answer, ask, first_diagnostics, next, wait_ready};

const SPEC: &str = "_NUnitApex/GreeterSpec.cs";
const WAIT: Duration = Duration::from_secs(180);
/// One deadline for the whole load (~42 s under load ~10), not one per event.
const LOAD: Duration = Duration::from_secs(300);

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path: PathBuf = entry.unwrap().path();
        let name: String = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() && name != "obj" && name != "bin" {
            copy_tree(&path, &to.join(&name));
        } else if path.is_file() {
            std::fs::copy(&path, to.join(&name)).unwrap();
        }
    }
}

fn harness(name: &str) -> PathBuf {
    let base: PathBuf = std::env::temp_dir().join(format!("bb-gui-{name}-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&base);
    let source: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/_Harness");
    copy_tree(&source, &base);
    std::fs::canonicalize(base).unwrap()
}

struct Live {
    link: Link,
    events: UnboundedReceiver<Event>,
    root: PathBuf,
    spec: PathBuf,
}

impl Live {
    async fn ask(&mut self, ticket: u64, question: Ask) -> (Answer, Stages) {
        let sent: Ticket = ask(&self.link, &self.spec, ticket, question);
        answer(&mut self.events, sent).await
    }

    async fn definitions(&mut self, ticket: u64, at: LspPoint) -> Vec<Place> {
        let (found, _): (Answer, Stages) = self.ask(ticket, Ask::Definition(at)).await;
        let Answer::Definition(places): Answer = found else {
            panic!("{found:?}");
        };
        places
    }
}

async fn loaded(name: &str) -> Live {
    let root: PathBuf = harness(name);
    let solution: PathBuf = find_solution(&root).unwrap();
    assert_eq!(solution, root.join("Harness.slnx"));
    let (link, mut events): (Link, UnboundedReceiver<Event>) = start(&solution);
    let progress: Vec<(usize, usize)> = wait_ready(&mut events).await;
    assert_eq!(progress.first(), Some(&(0, 3)), "{progress:?}");
    assert_eq!(progress.last(), Some(&(3, 3)), "{progress:?}");
    assert!(progress
        .windows(2)
        .all(|w: &[(usize, usize)]| w[0].0 <= w[1].0));
    let spec: PathBuf = root.join(SPEC);
    let text: String = std::fs::read_to_string(&spec).unwrap();
    assert!(link.send(Request::Open {
        path: spec.clone(),
        text,
    }));
    first_diagnostics(&mut events).await;
    Live {
        link,
        events,
        root,
        spec,
    }
}

#[tokio::test]
async fn the_server_thread_counts_projects_then_syncs_edits_and_answers() {
    let mut live: Live = loaded("loop").await;
    complete_after_an_edit(&mut live).await;
    navigate(&mut live).await;
    rename_changes_no_file(&mut live).await;
}

async fn complete_after_an_edit(live: &mut Live) {
    let text: String = std::fs::read_to_string(&live.spec).unwrap();
    let at: CharIdx = Buffer::from_text(&text)
        .from_lsp(LspPoint {
            line: 15,
            character: 0,
        })
        .unwrap();
    let typed: Edit = Edit {
        from: at,
        to: at,
        text: String::from("        greeter.\n"),
    };
    assert!(live.link.send(Request::Change {
        path: live.spec.clone(),
        edits: vec![typed],
    }));
    let dot: LspPoint = LspPoint {
        line: 15,
        character: 16,
    };
    let (completion, stages): (Answer, Stages) = live.ask(1, Ask::Completion(dot)).await;
    let Answer::Completion(items): Answer = completion else {
        panic!("{completion:?}");
    };
    assert!(
        items
            .iter()
            .any(|i: &bb_intel::CompletionEntry| i.label == "Hello"),
        "{items:?}"
    );
    assert!(stages.queued() < Duration::from_millis(500), "{stages:?}");
    assert!(stages.server() > Duration::ZERO);
    let errors: Vec<Diagnostic> = edited_diagnostics(live).await;
    assert!(
        errors
            .iter()
            .any(|d: &Diagnostic| d.severity == Some(Severity::Error)),
        "{errors:?}"
    );
}

/// The first diagnostics after the edit: for the edited text, never an
/// older list (BBEOD03-DIAGVER: one run once saw an empty batch).
async fn edited_diagnostics(live: &mut Live) -> Vec<Diagnostic> {
    loop {
        if let Event::Diagnostics {
            path,
            version,
            list,
        } = next(&mut live.events).await
        {
            assert_eq!(path, live.spec);
            assert_eq!(version.raw(), 2, "{list:?}");
            return list;
        }
    }
}

const CLASS_USE: LspPoint = LspPoint {
    line: 12,
    character: 9,
};

async fn navigate(live: &mut Live) {
    let places: Vec<Place> = live.definitions(2, CLASS_USE).await;
    let place: &Place = places.first().unwrap();
    let greeter: PathBuf = live.root.join("Core/Greeter.cs");
    assert!(matches!(&place.location.uri, DocUri::File(p) if p.as_path() == greeter));
    assert_eq!(place.preview, "public sealed class Greeter");
    let string_use: LspPoint = LspPoint {
        line: 13,
        character: 9,
    };
    let places: Vec<Place> = live.definitions(3, string_use).await;
    let target: &DocUri = &places.first().unwrap().location.uri;
    eprintln!("metadata definition uri: {target:?}");
    if let DocUri::File(path) = target {
        assert!(!path.as_path().starts_with(&live.root), "{target:?}");
        assert!(path.as_path().is_file(), "{target:?}");
    }
}

async fn rename_changes_no_file(live: &mut Live) {
    let greeter: PathBuf = live.root.join("Core/Greeter.cs");
    let before: String = std::fs::read_to_string(&greeter).unwrap();
    let (renamed, _): (Answer, Stages) = live
        .ask(4, Ask::Rename(CLASS_USE, String::from("Welcomer")))
        .await;
    let Answer::Rename(changes): Answer = renamed else {
        panic!("{renamed:?}");
    };
    assert_eq!(changes.len(), 2, "{changes:?}");
    assert!(changes
        .iter()
        .all(|c: &FileChange| matches!(c, FileChange::Edit { .. })));
    assert_eq!(std::fs::read_to_string(&greeter).unwrap(), before);
}
