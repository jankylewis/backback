use std::path::{Path, PathBuf};

use bb_intel::{CompletionEntry, Diagnostic, Severity, Span};
use bb_text::{CharIdx, LspPoint};

use super::popups::{CompletionPopup, Pick};
use super::{fits, freshness, Freshness, IntelModel, LoadStatus, Now, Pending, Question};
use crate::intel::bb_worker::protocol::{AskKind, Ticket};

const KINDS: [AskKind; 6] = [
    AskKind::Completion,
    AskKind::Hover,
    AskKind::Definition,
    AskKind::References,
    AskKind::Rename,
    AskKind::Actions,
];

fn point(line: u32, character: u32) -> LspPoint {
    LspPoint { line, character }
}

fn question(kind: AskKind, path: &str, revision: u64, anchor: usize) -> Question {
    Question {
        kind,
        path: PathBuf::from(path),
        revision,
        at: point(0, 0),
        anchor: CharIdx::new(anchor),
    }
}

fn now(active: &str, revision: Option<u64>, word_start: usize) -> Now {
    Now {
        active: Some(PathBuf::from(active)),
        revision,
        word_start: Some(CharIdx::new(word_start)),
    }
}

fn entry(label: &str) -> CompletionEntry {
    CompletionEntry {
        label: label.to_owned(),
        kind: None,
        detail: None,
    }
}

fn diag(line: u32, severity: Option<Severity>) -> Diagnostic {
    Diagnostic {
        span: Span {
            start: point(line, 1),
            end: point(line, 4),
        },
        severity,
        code: None,
        message: format!("line {line}"),
    }
}

#[test]
fn the_status_bar_counts_projects_until_ready() {
    let shown: Vec<String> = [
        LoadStatus::Loading {
            loaded: 0,
            total: 17,
        },
        LoadStatus::Loading {
            loaded: 3,
            total: 17,
        },
        LoadStatus::Ready,
        LoadStatus::NoSolution,
    ]
    .iter()
    .map(LoadStatus::label)
    .collect();
    assert_eq!(
        shown,
        [
            "Loading 0/17 projects",
            "Loading 3/17 projects",
            "Ready",
            "No solution"
        ]
    );
}

#[test]
fn only_the_newest_question_of_each_kind_is_answered() {
    for kind in KINDS {
        let mut model: IntelModel = IntelModel::new(LoadStatus::Ready);
        let old: Ticket = model.in_flight.ask(question(kind, "/a.cs", 1, 4));
        let new: Ticket = model.in_flight.ask(question(kind, "/a.cs", 1, 4));
        let here: Now = now("/a.cs", Some(1), 4);
        assert_eq!(model.in_flight.accept(old, &here), None, "{kind:?}");
        let taken: Option<Pending> = model.in_flight.accept(new, &here);
        assert_eq!(taken.map(|p: Pending| p.ticket), Some(new), "{kind:?}");
        assert!(!model.in_flight.waiting(kind));
    }
}

#[test]
fn an_answer_about_changed_text_is_dropped_by_its_kind_rule() {
    for kind in KINDS {
        let mut model: IntelModel = IntelModel::new(LoadStatus::Ready);
        let ticket: Ticket = model.in_flight.ask(question(kind, "/a.cs", 1, 4));
        let typed: Now = now("/a.cs", Some(2), 4);
        let kept: bool = model.in_flight.accept(ticket, &typed).is_some();
        let expected: bool = freshness(kind) != Freshness::SameText;
        assert_eq!(kept, expected, "{kind:?}");
    }
}

#[test]
fn a_completion_fits_only_while_the_cursor_stays_in_its_word() {
    let pending: Pending = Pending {
        ticket: Ticket(1),
        kind: AskKind::Completion,
        path: PathBuf::from("/a.cs"),
        revision: 1,
        at: point(0, 4),
        anchor: CharIdx::new(4),
        sent: std::time::Instant::now(),
    };
    let rule: Freshness = Freshness::SameWord;
    assert!(fits(&pending, rule, &now("/a.cs", Some(5), 4)));
    assert!(!fits(&pending, rule, &now("/a.cs", Some(5), 9)));
    assert!(!fits(&pending, rule, &now("/b.cs", Some(1), 4)));
    assert!(!fits(&pending, Freshness::SameText, &now("/a.cs", None, 4)));
}

#[test]
fn completion_filters_by_prefix_then_by_contains_and_keeps_the_pick_inside() {
    let items: Vec<CompletionEntry> = ["WriteLine", "Hello", "ToString", "help", "Shell"]
        .into_iter()
        .map(entry)
        .collect();
    let mut popup: CompletionPopup =
        CompletionPopup::new(PathBuf::from("/a"), CharIdx::new(0), items);
    popup.set_filter("he");
    let labels: Vec<&str> = popup
        .visible()
        .iter()
        .map(|e: &&CompletionEntry| e.label.as_str())
        .collect();
    assert_eq!(labels, ["Hello", "help", "Shell"]);
    popup.step(true);
    popup.step(true);
    popup.step(true);
    assert_eq!(popup.chosen().as_deref(), Some("Shell"));
    popup.set_filter("hel");
    assert_eq!(popup.pick, Pick::default());
    popup.set_filter("zz");
    assert_eq!(popup.chosen(), None);
}

#[test]
fn problems_sort_by_file_line_and_severity_and_leave_with_their_tab() {
    let mut model: IntelModel = IntelModel::new(LoadStatus::Ready);
    model.set_diagnostics(
        PathBuf::from("/b.cs"),
        vec![diag(7, Some(Severity::Warning)), diag(2, None)],
    );
    model.set_diagnostics(
        PathBuf::from("/a.cs"),
        vec![
            diag(3, Some(Severity::Hint)),
            diag(3, Some(Severity::Error)),
        ],
    );
    let order: Vec<(String, String)> = model
        .problems()
        .iter()
        .map(|(path, d): &(&Path, &Diagnostic)| (path.display().to_string(), d.message.clone()))
        .collect();
    assert_eq!(order.len(), 4);
    assert_eq!(
        model
            .problems()
            .first()
            .map(|p: &(&Path, &Diagnostic)| p.1.severity),
        Some(Some(Severity::Error))
    );
    assert_eq!(
        order.last().map(|p: &(String, String)| p.1.as_str()),
        Some("line 7")
    );
    model.in_flight.ask(question(AskKind::Hover, "/a.cs", 1, 0));
    model.closed(Path::new("/a.cs"));
    assert_eq!(
        model.diagnostics_of(Path::new("/a.cs")),
        Vec::<Diagnostic>::new().as_slice()
    );
    assert!(!model.in_flight.waiting(AskKind::Hover));
    assert_eq!(model.problems().len(), 2);
}

#[test]
fn a_late_batch_line_never_turns_ready_or_failed_back_into_loading() {
    let mut status: LoadStatus = LoadStatus::Loading {
        loaded: 0,
        total: 4,
    };
    status.progress(4, 4);
    assert_eq!(status.label(), "Loading 4/4 projects");
    status = LoadStatus::Ready;
    status.progress(4, 4);
    assert_eq!(status, LoadStatus::Ready);
    status = LoadStatus::Failed(String::from("x"));
    status.progress(1, 4);
    assert_eq!(status, LoadStatus::Failed(String::from("x")));
}

#[test]
fn problems_list_only_files_inside_the_workspace() {
    let mut model: IntelModel = IntelModel::new(LoadStatus::Ready);
    model.set_diagnostics(
        PathBuf::from("/ws/a.cs"),
        vec![diag(1, Some(Severity::Error))],
    );
    model.set_diagnostics(PathBuf::from("/tmp/meta/String.cs"), vec![diag(2, None)]);
    let listed: Vec<String> = model
        .problems_in(Path::new("/ws"))
        .iter()
        .map(|(path, _): &(&Path, &Diagnostic)| path.display().to_string())
        .collect();
    assert_eq!(listed, ["/ws/a.cs"]);
    assert_eq!(
        model.diagnostics_of(Path::new("/tmp/meta/String.cs")).len(),
        1
    );
}
