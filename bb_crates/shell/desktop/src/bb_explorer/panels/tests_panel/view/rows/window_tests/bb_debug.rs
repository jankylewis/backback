//! Debugging a test from where it is: a red ■ on the row of the test being
//! debugged stops it (BB-FEAT-081); a right-click inside a test method in
//! the editor opens Run and Debug for that test (BB-FEAT-084).

use bb_testkit::Fqn;
use gpui::{Modifiers, MouseButton, TestAppContext, Window};
use tokio::sync::mpsc::UnboundedReceiver;

use super::{click, shown, tree_window, Rig};
use crate::run::bb_debug::session::Command;
use crate::ExplorerView;

#[gpui::test]
fn the_debugged_tests_row_shows_a_stop_that_ends_the_session(cx: &mut TestAppContext) {
    let (_dir, _project, view, vcx): Rig<'_> = tree_window(cx);
    let mut sent: UnboundedReceiver<Command> = view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            v.debug.test = Some(Fqn::new("Sample.Auth.OutcomeSpec.Passes").unwrap());
            cx.notify();
            crate::run::bb_debug::session::control::fake_paused(v, Vec::new(), None)
        },
    );
    vcx.run_until_parked();
    assert!(shown(vcx, "test-debug-stop-Sample.Auth.OutcomeSpec.Passes"));
    assert!(
        !shown(vcx, "test-debug-stop-Sample.Auth.OutcomeSpec.Fails"),
        "only the debugged test"
    );
    click(vcx, "test-debug-stop-Sample.Auth.OutcomeSpec.Passes");
    assert_eq!(format!("{:?}", sent.try_recv()), "Ok(Stop)");
}

#[gpui::test]
fn a_right_click_in_a_test_method_in_the_editor_offers_run_and_debug(cx: &mut TestAppContext) {
    let (dir, _project, view, vcx): Rig<'_> = tree_window(cx);
    let file: std::path::PathBuf = dir.path().canonicalize().unwrap().join("OutcomeSpec.cs");
    vcx.update(|window: &mut Window, cx: &mut gpui::App| {
        view.update(
            cx,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                v.tabs.set_no_write();
                v.open_file(&file, window, cx);
            },
        );
    });
    assert!(
        parsed(&view, vcx),
        "the spec file's test methods were found"
    );
    // BB-BUG-065: the gutter's ▶ runs its test (it opened the panel before:
    // the full name never equalled the method's own name).
    view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            v.tests.message = None;
        },
    );
    click(vcx, "test-gutter-5");
    let ran: bool = view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tests.message.is_some() || v.tests.busy.is_some()
    });
    assert!(ran, "the gutter's ▶ ran the test");
    let line: gpui::Point<gpui::Pixels> = vcx.debug_bounds("editor-line-5").unwrap().center();
    vcx.simulate_mouse_down(line, MouseButton::Right, Modifiers::default());
    vcx.run_until_parked();
    let picked: Option<String> = view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tests
            .menu
            .as_ref()
            .map(|m: &crate::tests_panel::TestMenu| m.fqn.as_str().to_owned())
    });
    assert_eq!(picked.as_deref(), Some("Sample.Auth.OutcomeSpec.Fails"));
    assert!(shown(vcx, "test-menu-run") && shown(vcx, "test-menu-debug"));
    click(vcx, "test-menu-debug");
    let said: Option<String> =
        view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.status.clone());
    assert!(
        said.is_some_and(|s: String| s.starts_with("Debug refused")),
        "Debug ran"
    );
}

/// BB-BUG-074: the menu opens only inside a test method (its attribute
/// through its body): a right-click on the class's brace after the last
/// test opens nothing, and nothing else opens instead.
#[gpui::test]
fn a_right_click_outside_a_test_method_opens_no_test_menu(cx: &mut TestAppContext) {
    let (dir, _project, view, vcx): Rig<'_> = tree_window(cx);
    let file: std::path::PathBuf = dir.path().canonicalize().unwrap().join("OutcomeSpec.cs");
    vcx.update(|window: &mut Window, cx: &mut gpui::App| {
        view.update(
            cx,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                v.open_file(&file, window, cx);
            },
        );
    });
    assert!(parsed(&view, vcx));
    assert_eq!(menu(&view, vcx, 7), None, "the class's closing brace");
    assert_eq!(menu(&view, vcx, 3), None, "the class's opening brace");
    assert_eq!(
        menu(&view, vcx, 4).as_deref(),
        Some("Sample.Auth.OutcomeSpec.Passes")
    );
}

/// Right-click line `line`; the test the menu opened for, closing it.
fn menu(
    view: &gpui::Entity<ExplorerView>,
    vcx: &mut gpui::VisualTestContext,
    line: usize,
) -> Option<String> {
    let at: gpui::Point<gpui::Pixels> = vcx
        .debug_bounds(format!("editor-line-{line}").leak())
        .unwrap()
        .center();
    vcx.simulate_mouse_down(at, MouseButton::Right, Modifiers::default());
    vcx.run_until_parked();
    view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            v.tests
                .menu
                .take()
                .map(|m: crate::tests_panel::TestMenu| m.fqn.as_str().to_owned())
        },
    )
}

/// Whether the open spec file's test methods were found (the parse runs
/// off the UI thread).
fn parsed(view: &gpui::Entity<ExplorerView>, vcx: &mut gpui::VisualTestContext) -> bool {
    for _ in 0..200 {
        vcx.run_until_parked();
        let found: bool = view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            v.tabs
                .active_tab()
                .is_some_and(|t: &crate::editor::Tab| t.test_methods.len() >= 2)
        });
        if found {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    false
}
