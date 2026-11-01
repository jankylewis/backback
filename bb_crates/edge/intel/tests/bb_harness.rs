//! Every feature against the real roslyn-language-server on the harness.

#[cfg(test)]
mod common;

use bb_intel::{
    ActionItem, CompletionEntry, Diagnostic, DocPath, DocUri, Edit, FileChange, HoverInfo,
    IntelError, Location, Severity, Span,
};
use bb_text::{Buffer, CharIdx, LspPoint};
use common::{start, Harness, DEADLINE};

const SPEC: &str = "_NUnitApex/GreeterSpec.cs";
const GREETER_USE: LspPoint = LspPoint {
    line: 12,
    character: 9,
};

fn file_of(uri: &DocUri) -> String {
    match uri {
        DocUri::File(p) => p
            .as_path()
            .file_name()
            .map_or_else(String::new, |n: &std::ffi::OsStr| {
                n.to_string_lossy().into_owned()
            }),
        DocUri::Virtual(v) => v.clone(),
    }
}

#[tokio::test]
async fn definition_references_and_hover_reach_across_projects() {
    let mut h: Harness = start("nav").await;
    let spec: DocPath = h.doc(SPEC);
    h.session.open(&spec).await.unwrap();
    let defs: Vec<Location> = h
        .session
        .definition(&spec, GREETER_USE, DEADLINE)
        .await
        .unwrap();
    let refs: Vec<Location> = h
        .session
        .references(&spec, GREETER_USE, DEADLINE)
        .await
        .unwrap();
    let hello: LspPoint = LspPoint {
        line: 13,
        character: 33,
    };
    let hover: Option<HoverInfo> = h.session.hover(&spec, hello, DEADLINE).await.unwrap();
    h.finish().await;
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(file_of(&defs[0].uri), "Greeter.cs");
    assert_eq!(
        defs[0].span.start,
        LspPoint {
            line: 2,
            character: 20
        }
    );
    let mut files: Vec<String> = refs.iter().map(|l: &Location| file_of(&l.uri)).collect();
    files.sort();
    files.dedup();
    assert_eq!(files, ["Greeter.cs", "GreeterSpec.cs"], "{refs:?}");
    assert_eq!(refs.len(), 3, "{refs:?}");
    assert!(hover.unwrap().text.contains("Greeter.Hello(string name)"));
}

#[tokio::test]
async fn rename_returns_edits_in_two_files_and_applies_none() {
    let mut h: Harness = start("rename").await;
    let spec: DocPath = h.doc(SPEC);
    let before: String = std::fs::read_to_string(h.root.join("Core/Greeter.cs")).unwrap();
    h.session.open(&spec).await.unwrap();
    let plan: Vec<FileChange> = h
        .session
        .rename(&spec, GREETER_USE, "Welcomer", DEADLINE)
        .await
        .unwrap();
    let after: String = std::fs::read_to_string(h.root.join("Core/Greeter.cs")).unwrap();
    let text: Option<String> = h.session.text(&spec);
    h.finish().await;
    let mut files: Vec<String> = plan.iter().map(|c: &FileChange| file_of(c.uri())).collect();
    files.sort();
    assert_eq!(files, ["Greeter.cs", "GreeterSpec.cs"], "{plan:?}");
    assert!(plan.iter().all(
        |c: &FileChange| matches!(c, FileChange::Edit { changes, .. } if !changes.is_empty())
    ));
    assert_eq!(after, before);
    assert!(text.unwrap().contains("Greeter greeter = new Greeter();"));
}

#[tokio::test]
async fn completion_after_an_incremental_edit_lists_hello() {
    let mut h: Harness = start("complete").await;
    let spec: DocPath = h.doc(SPEC);
    h.session.open(&spec).await.unwrap();
    let mirror: Buffer = Buffer::from_text(&h.session.text(&spec).unwrap());
    let at: CharIdx = mirror
        .from_lsp(LspPoint {
            line: 14,
            character: 0,
        })
        .unwrap();
    let typed: Edit = Edit {
        from: at,
        to: at,
        text: "        greeter.\n".to_owned(),
    };
    let version: bb_intel::DocVersion = h.session.change(&spec, &[typed]).await.unwrap();
    let dot: LspPoint = LspPoint {
        line: 14,
        character: 16,
    };
    let items: Vec<CompletionEntry> = h.session.completion(&spec, dot, DEADLINE).await.unwrap();
    h.finish().await;
    assert_eq!(version.raw(), 2);
    let labels: Vec<&str> = items
        .iter()
        .map(|i: &CompletionEntry| i.label.as_str())
        .collect();
    assert!(labels.contains(&"Hello"), "{labels:?}");
    assert!(!labels.contains(&"Greets"), "{labels:?}");
}

#[tokio::test]
async fn a_broken_file_yields_an_error_and_code_actions_are_listed() {
    let mut h: Harness = start("broken").await;
    let broken: DocPath = h.doc("Core/Broken.cs");
    let text: &str = "namespace Harness.Core;\nclass B { void M() { int x = } }\n";
    h.session.open_text(&broken, text).await.unwrap();
    let found: Vec<Diagnostic> = h.session.diagnostics(&broken, DEADLINE).await.unwrap();
    let kept: Vec<Diagnostic> = h.session.collected(&broken);
    let spec: DocPath = h.doc(SPEC);
    h.session.open(&spec).await.unwrap();
    let call: Span = Span {
        start: LspPoint {
            line: 13,
            character: 24,
        },
        end: LspPoint {
            line: 13,
            character: 43,
        },
    };
    let actions: Vec<ActionItem> = h.session.code_actions(&spec, call, DEADLINE).await.unwrap();
    h.finish().await;
    let errors: Vec<&Diagnostic> = found
        .iter()
        .filter(|d: &&Diagnostic| d.severity == Some(Severity::Error))
        .collect();
    assert!(
        errors
            .iter()
            .any(|d: &&Diagnostic| d.code.as_deref() == Some("CS1525")),
        "{found:?}"
    );
    assert_eq!(errors[0].span.start.line, 1);
    assert_eq!(kept, found);
    assert!(
        actions.iter().any(|a: &ActionItem| a
            .kind
            .as_deref()
            .is_some_and(|k: &str| k.starts_with("refactor"))),
        "{actions:?}"
    );
}

#[tokio::test]
async fn features_refuse_a_document_that_is_not_open() {
    let mut h: Harness = start("notopen").await;
    let spec: DocPath = h.doc(SPEC);
    let early: Result<Vec<Location>, IntelError> =
        h.session.definition(&spec, GREETER_USE, DEADLINE).await;
    h.session.open(&spec).await.unwrap();
    let twice: Result<Vec<Diagnostic>, IntelError> = h.session.open(&spec).await;
    h.session.close(&spec).await.unwrap();
    let closed: Result<Vec<CompletionEntry>, IntelError> =
        h.session.completion(&spec, GREETER_USE, DEADLINE).await;
    h.finish().await;
    assert!(matches!(early, Err(IntelError::NotOpen(_))), "{early:?}");
    assert!(
        matches!(twice, Err(IntelError::AlreadyOpen(_))),
        "{twice:?}"
    );
    assert!(matches!(closed, Err(IntelError::NotOpen(_))), "{closed:?}");
}
