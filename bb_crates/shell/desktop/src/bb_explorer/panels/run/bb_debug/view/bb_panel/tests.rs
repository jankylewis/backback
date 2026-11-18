//! Looking into values while paused, at the window: the Variables tree
//! opens a row and keeps it open across stops (BB-FEAT-087); a resting
//! mouse shows a name's value (BB-FEAT-088); a debugged test's results mark
//! its row (BB-BUG-066).

use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::run::bb_debug::session::control::fake_paused;
use crate::run::bb_debug::session::{Command, Frame, Note, Var};
use crate::ExplorerView;

fn var(name: &str, value: &str, reference: i64) -> Var {
    Var {
        name: name.to_owned(),
        value: value.to_owned(),
        reference,
    }
}

pub(super) fn frame(rig: &Rig) -> Frame {
    Frame {
        id: 1,
        name: String::from("A.M()"),
        path: Some(rig.file.clone()),
        line: 0,
    }
}

pub(super) fn note(rig: &Rig, vcx: &mut VisualTestContext, note: Note) {
    vcx.update(|window: &mut gpui::Window, cx: &mut gpui::App| {
        rig.view.update(
            cx,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                super::super::notes::apply(v, note, window, cx);
                cx.notify();
            },
        );
    });
    vcx.run_until_parked();
}

/// Paused with `vars` as the top frame's; the commands the window sends.
pub(super) fn paused(
    rig: &Rig,
    vcx: &mut VisualTestContext,
    vars: Vec<Var>,
) -> UnboundedReceiver<Command> {
    let top: Frame = frame(rig);
    let sent: UnboundedReceiver<Command> = rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let sent: UnboundedReceiver<Command> = fake_paused(v, vec![top], None);
            cx.notify();
            sent
        },
    );
    note(rig, vcx, Note::Variables { frame: 1, vars });
    sent
}

fn sent(commands: &mut UnboundedReceiver<Command>) -> Vec<Command> {
    std::iter::from_fn(|| commands.try_recv().ok()).collect()
}

fn click(vcx: &mut VisualTestContext, selector: &'static str) {
    let at: Bounds<Pixels> = vcx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is drawn"));
    vcx.simulate_click(at.center(), Modifiers::default());
    vcx.run_until_parked();
}

#[gpui::test]
fn a_click_opens_a_variable_and_its_children_show_under_it(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    let roots: Vec<Var> = vec![var("in_name", "\"Seed\"", 0), var("in_body", "{Func}", 7)];
    let mut commands: UnboundedReceiver<Command> = paused(&rig, vcx, roots.clone());
    assert!(vcx.debug_bounds("debug-var-in_body").is_some());
    click(vcx, "debug-var-in_name");
    assert_eq!(
        sent(&mut commands),
        Vec::new(),
        "a plain value has no children"
    );
    click(vcx, "debug-var-in_body");
    assert_eq!(sent(&mut commands), vec![Command::Expand(7)]);
    let kids: Vec<Var> = vec![var("Target", "{Spec}", 9), var("Method", "{Void M()}", 0)];
    note(
        &rig,
        vcx,
        Note::Children {
            reference: 7,
            vars: kids,
        },
    );
    let parent: Bounds<Pixels> = vcx.debug_bounds("debug-var-in_body").unwrap();
    let child: Bounds<Pixels> = vcx.debug_bounds("debug-var-in_body/Target").unwrap();
    assert!(child.origin.y > parent.origin.y, "under its parent");
    // A step: the program runs, stops again; the row stays open and its
    // children are asked again (references end at each stop).
    note(&rig, vcx, Note::Running);
    let again: Note = Note::Stopped {
        reason: String::from("step"),
        exception: None,
        frames: vec![frame(&rig)],
        vars: vec![var("in_name", "\"Seed\"", 0), var("in_body", "{Func}", 7)],
    };
    note(&rig, vcx, again);
    // The adapter numbers references anew at each stop: 7 again is a
    // new 7, asked again.
    assert_eq!(sent(&mut commands), vec![Command::Expand(7)]);
    click(vcx, "debug-var-in_body");
    assert!(
        vcx.debug_bounds("debug-var-in_body/Target").is_none(),
        "closed"
    );
}

#[gpui::test]
fn a_resting_mouse_on_a_name_asks_the_debugger_and_shows_its_value(cx: &mut TestAppContext) {
    let text: &str = "class A { void M(string in_name) { Use(in_name); } }\n";
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, text);
    let mut commands: UnboundedReceiver<Command> = paused(&rig, vcx, Vec::new());
    let at: bb_text::CharIdx = bb_text::CharIdx::new(text.rfind("in_name").unwrap() + 2);
    let token: u64 = 5;
    let file: std::path::PathBuf = rig.file.clone();
    let asked: bool = rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            v.intel.hover_target = Some((file.clone(), at, token));
            crate::run::bb_debug::view::inspect::hover_value(v, &file, at, token)
        },
    );
    assert!(asked, "paused in this file: the debugger is asked");
    assert_eq!(
        sent(&mut commands),
        vec![Command::Evaluate {
            expression: String::from("in_name"),
            frame: 1,
            token,
        }]
    );
    let value: Result<Var, String> = Ok(var("in_name", "\"Seed 2 chips\"", 0));
    note(&rig, vcx, Note::Evaluated { token, value });
    let shown: Option<String> = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.intel
            .hover
            .as_ref()
            .map(|h: &crate::intel::model::popups::HoverPopup| h.text.clone())
    });
    assert_eq!(shown.as_deref(), Some("in_name = \"Seed 2 chips\""));
    let stale: Result<Var, String> = Ok(var("in_name", "\"old\"", 0));
    note(
        &rig,
        vcx,
        Note::Evaluated {
            token: 4,
            value: stale,
        },
    );
    let kept: Option<String> = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.intel
            .hover
            .as_ref()
            .map(|h: &crate::intel::model::popups::HoverPopup| h.text.clone())
    });
    assert_eq!(kept, shown, "an answer for an older hover is dropped");
}

#[gpui::test]
fn a_debugged_tests_results_mark_its_row_after_the_session_ends(cx: &mut TestAppContext) {
    use bb_testkit::{Fqn, Outcome, SpecResult};
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    let project: std::path::PathBuf = rig.root.join("T.csproj");
    let name: &str = "N.C.Passes";
    let fqn: Fqn = Fqn::new(name).unwrap();
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            let list: bb_testkit::SpecList = bb_testkit::SpecList {
                project: String::from("T"),
                assembly: std::path::PathBuf::from("T.dll"),
                specs: vec![Fqn::new(name).unwrap()],
                elapsed: std::time::Duration::ZERO,
            };
            v.tests.model.merge(&project, list);
        },
    );
    let _commands: UnboundedReceiver<Command> = paused(&rig, vcx, Vec::new());
    note(&rig, vcx, Note::Ended(String::from("the program ended")));
    let result: SpecResult = SpecResult {
        fqn,
        outcome: Outcome::Failed,
        duration: std::time::Duration::from_millis(12),
        message: Some(String::from("boom")),
        stack: None,
    };
    let results: Vec<SpecResult> = vec![result];
    note(
        &rig,
        vcx,
        Note::TestResults {
            project: project.clone(),
            results,
        },
    );
    let outcome: Option<Outcome> = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tests.model.outcome(&project, name)
    });
    assert_eq!(outcome, Some(Outcome::Failed), "red, as after a run");
}
