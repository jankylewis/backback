//! F12, Shift+F12, Cmd+., Problems and the status bar.

use std::path::PathBuf;
use std::time::Duration;

use bb_intel::{ActionItem, Diagnostic, Severity, Span};
use bb_text::LspPoint;

use super::bb_rig::place;
use crate::intel::bb_worker::protocol::{Answer, Ask, Event, Ticket};
use gpui::{TestAppContext, VisualTestContext};

use super::bb_rig::{rig, Rig};
use crate::ExplorerView;

#[gpui::test]
fn f12_opens_the_target_and_metadata_outside_the_root_is_read_only(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    let outside: tempfile::TempDir = tempfile::tempdir().unwrap();
    let meta: PathBuf = outside.path().canonicalize().unwrap().join("Meta.cs");
    std::fs::write(&meta, "line0\nclass Meta {}\n").unwrap();
    vcx.simulate_keystrokes("f12");
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::Definition(_)));
    rig.reply(vcx, ticket, Answer::Definition(vec![place(&meta, 1)]));
    vcx.simulate_keystrokes("q backspace");
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
        assert_eq!(tab.path, meta);
        assert!(tab.read_only);
        assert_eq!(tab.document.cursor_line(), 1);
        assert_eq!(tab.document.text(), "line0\nclass Meta {}\n");
        assert_eq!(v.status.as_deref(), Some("read-only file: edit refused"));
    });
}

#[gpui::test]
fn shift_f12_lists_references_and_escape_closes_the_panel(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.simulate_keystrokes("shift-f12");
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::References(_)));
    let file: PathBuf = rig.file.clone();
    rig.reply(
        vcx,
        ticket,
        Answer::References(vec![place(&file, 0), place(&file, 0)]),
    );
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(
            v.intel
                .references
                .as_ref()
                .map(|p: &crate::intel::model::popups::PlacesPanel| p.title.as_str()),
            Some("2 references")
        );
    });
    vcx.simulate_keystrokes("escape");
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert!(v.intel.references.is_none());
    });
}

#[gpui::test]
fn the_status_bar_follows_progress_until_ready(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    rig.events
        .send(Event::Progress {
            loaded: 1,
            total: 4,
        })
        .unwrap();
    vcx.run_until_parked();
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(v.intel.status.label(), "Loading 1/4 projects");
    });
    rig.events
        .send(Event::Ready {
            took: Duration::ZERO,
        })
        .unwrap();
    vcx.run_until_parked();
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(v.intel.status.label(), "Ready");
    });
}

#[gpui::test]
fn cmd_dot_lists_actions_and_diagnostics_reach_problems(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    let error: Diagnostic = Diagnostic {
        span: Span {
            start: LspPoint {
                line: 0,
                character: 21,
            },
            end: LspPoint {
                line: 0,
                character: 22,
            },
        },
        severity: Some(Severity::Error),
        code: Some(String::from("CS0103")),
        message: String::from("The name 'x' does not exist"),
    };
    rig.events
        .send(Event::Diagnostics {
            path: rig.file.clone(),
            list: vec![error],
        })
        .unwrap();
    vcx.simulate_keystrokes("cmd-shift-m cmd-.");
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::Actions(..)));
    let action: ActionItem = ActionItem {
        title: String::from("Generate variable 'x'"),
        kind: None,
    };
    rig.reply(vcx, ticket, Answer::Actions(vec![action]));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert!(v.intel.problems_open);
        assert_eq!(v.intel.problems().len(), 1);
        assert_eq!(
            v.intel
                .actions
                .as_ref()
                .map(|a: &crate::intel::model::popups::ActionsPopup| a.items.len()),
            Some(1)
        );
    });
    vcx.simulate_keystrokes("escape");
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert!(v.intel.actions.is_none());
    });
}
