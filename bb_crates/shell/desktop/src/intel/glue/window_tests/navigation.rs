//! F12, Shift+F12, Cmd+., Problems and the status bar.

use std::path::PathBuf;
use std::time::Duration;

use bb_intel::{ActionItem, Diagnostic, Severity, Span};
use bb_text::LspPoint;

use super::bb_rig::{place, rig_with};
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
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, false, super::bb_rig::A);
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

#[gpui::test]
fn a_progress_event_after_ready_keeps_ready_and_questions_still_go(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    rig.events
        .send(Event::Progress {
            loaded: 4,
            total: 4,
        })
        .unwrap();
    vcx.run_until_parked();
    vcx.simulate_keystrokes("f12");
    let (_, ask): (Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::Definition(_)));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(v.intel.status.label(), "Ready");
    });
}

#[gpui::test]
fn long_problem_rows_never_squeeze_the_explorer(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    let long: Diagnostic = Diagnostic {
        span: Span {
            start: LspPoint {
                line: 0,
                character: 0,
            },
            end: LspPoint {
                line: 0,
                character: 1,
            },
        },
        severity: Some(Severity::Error),
        code: None,
        message: "x".repeat(600),
    };
    rig.events
        .send(Event::Diagnostics {
            path: rig.file.clone(),
            list: vec![long],
        })
        .unwrap();
    vcx.run_until_parked();
    vcx.simulate_keystrokes("cmd-shift-m");
    let left: f32 = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        f32::from(v.editor_scroll.0.borrow().base_handle.bounds().origin.x)
    });
    assert!(
        left >= 300.0,
        "editor starts at {left}, Explorer is 300 wide"
    );
}

fn rest_mouse(rig: &Rig, vcx: &mut VisualTestContext, at: usize) {
    rig.view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            let char_at: Option<bb_text::CharIdx> = Some(bb_text::CharIdx::new(at));
            crate::intel::glue::hover_move(v, char_at, (400.0, 60.0), window, cx);
        },
    );
    vcx.executor().advance_clock(Duration::from_millis(400));
    vcx.run_until_parked();
}

#[gpui::test]
fn a_hover_answer_for_a_char_the_mouse_left_shows_nothing(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    rest_mouse(&rig, vcx, 21);
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert_eq!(
        ask,
        Ask::Hover(LspPoint {
            line: 0,
            character: 21
        })
    );
    rig.view.update_in(
        vcx,
        |v: &mut ExplorerView, window: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::intel::glue::hover_move(
                v,
                Some(bb_text::CharIdx::new(6)),
                (90.0, 60.0),
                window,
                cx,
            );
        },
    );
    let info: bb_intel::HoverInfo = bb_intel::HoverInfo {
        text: String::from("local x"),
        span: None,
    };
    rig.reply(vcx, ticket, Answer::Hover(Some(info.clone())));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert!(v.intel.hover.is_none());
    });
    rest_mouse(&rig, vcx, 21);
    let (ticket, _): (Ticket, Ask) = rig.last_ask();
    rig.reply(vcx, ticket, Answer::Hover(Some(info)));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(
            v.intel
                .hover
                .as_ref()
                .map(|h: &crate::intel::model::popups::HoverPopup| h.text.as_str()),
            Some("local x")
        );
    });
}
