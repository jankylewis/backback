//! BBEOD27 panels with real cargo on the fixture workspace, by mouse and
//! keys: the Cargo explorer lists both members and opens one; Cmd+Shift+B in
//! a Rust window builds and lists the compile error, a click opens its
//! line; the Tests panel lists the three tests, runs one and the module,
//! shows pass, fail and ignored, and offers no Debug on a Rust row.

use std::path::PathBuf;

use bb_cargo::fixtures::{Shape, BROKEN_LINE};
use bb_lang::Tool;
use bb_testkit::Outcome;
use gpui::{App, Modifiers, TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::close_all;
use super::panel_kit::{
    click, members_read, outcome, right_click, rust_window, shown, wait_until, workspace, FAILS,
    IGNORED, PASSES,
};
use crate::ExplorerView;

#[gpui::test]
fn the_cargo_explorer_lists_both_members_and_a_click_opens_one(cx: &mut TestAppContext) {
    let (_base, root): (tempfile::TempDir, PathBuf) = workspace(Shape::Clean);
    let (window, mut vcx): (WindowHandle<ExplorerView>, VisualTestContext) = rust_window(cx, &root);
    wait_until(&mut vcx, (window, "cargo members"), members_read);
    let started: Vec<Tool> = window
        .read_with(&vcx, |v: &ExplorerView, _: &App| v.langs.started.clone())
        .unwrap();
    assert_eq!(started, vec![Tool::RustAnalyzer, Tool::Cargo]);
    assert!(shown(&mut vcx, "cargo-member-alpha"));
    assert!(shown(&mut vcx, "cargo-member-beta"));
    click(&mut vcx, "cargo-member-beta", Modifiers::default());
    let active: Option<PathBuf> = window
        .read_with(&vcx, |v: &ExplorerView, _: &App| {
            v.tabs
                .active_tab()
                .map(|t: &crate::editor::Tab| t.path.clone())
        })
        .unwrap();
    assert_eq!(active, Some(root.join("beta/Cargo.toml")));
    assert!(!root.join("target").exists());
    close_all(cx);
}

#[gpui::test]
fn cmd_shift_b_in_a_rust_window_lists_the_compile_error_and_a_click_opens_its_line(
    cx: &mut TestAppContext,
) {
    let (_base, root): (tempfile::TempDir, PathBuf) = workspace(Shape::Broken);
    let (window, mut vcx): (WindowHandle<ExplorerView>, VisualTestContext) = rust_window(cx, &root);
    wait_until(&mut vcx, (window, "cargo members"), members_read);
    click(&mut vcx, "cargo-member-beta", Modifiers::default());
    click(&mut vcx, "editor-area", Modifiers::default());
    vcx.update(|_: &mut gpui::Window, cx: &mut App| crate::launch::bind_keys(cx));
    vcx.simulate_keystrokes("cmd-shift-b");
    wait_until(&mut vcx, (window, "the build"), |v: &ExplorerView| {
        v.status
            .as_deref()
            .is_some_and(|s: &str| s.starts_with("Build "))
    });
    let (title, error_at): (String, Option<usize>) = window
        .read_with(&vcx, |v: &ExplorerView, _: &App| {
            let panel: &crate::intel::model::popups::PlacesPanel =
                v.intel.references.as_ref().unwrap();
            let at: Option<usize> =
                panel
                    .places
                    .iter()
                    .position(|p: &crate::intel::bb_worker::protocol::Place| {
                        p.preview.starts_with("error")
                    });
            (panel.title.clone(), at)
        })
        .unwrap();
    assert!(title.starts_with("Build failed: 1 error"), "{title}");
    click(
        &mut vcx,
        &format!("place-{}", error_at.unwrap()),
        Modifiers::default(),
    );
    let (path, line): (PathBuf, usize) = window
        .read_with(&vcx, |v: &ExplorerView, _: &App| {
            let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
            (tab.path.clone(), tab.document.cursor_line())
        })
        .unwrap();
    assert_eq!(
        (path, line),
        (
            root.join("beta/src/lib.rs"),
            usize::try_from(BROKEN_LINE - 1).unwrap()
        )
    );
    close_all(cx);
}

fn list_alpha_and_open_its_module(vcx: &mut VisualTestContext, window: WindowHandle<ExplorerView>) {
    wait_until(vcx, (window, "cargo members"), members_read);
    click(vcx, "open-tests", Modifiers::default());
    wait_until(vcx, (window, "the test scan"), |v: &ExplorerView| {
        v.tests.scan == crate::tests_panel::Scan::Done
    });
    click(vcx, "test-project-alpha", Modifiers::default());
    wait_until(vcx, (window, "the test list"), |v: &ExplorerView| {
        v.tests
            .model
            .projects()
            .values()
            .any(|p: &crate::tests_panel::model::ProjectTests| p.specs.len() == 3)
    });
    click(vcx, "test-ns-alpha", Modifiers::default());
    click(vcx, "test-class-alpha.tests", Modifiers::default());
}

#[gpui::test]
fn the_tests_panel_lists_rust_tests_runs_one_and_the_module_and_offers_no_debug(
    cx: &mut TestAppContext,
) {
    let (_base, root): (tempfile::TempDir, PathBuf) = workspace(Shape::Clean);
    let (window, mut vcx): (WindowHandle<ExplorerView>, VisualTestContext) = rust_window(cx, &root);
    list_alpha_and_open_its_module(&mut vcx, window);
    assert!(shown(&mut vcx, &format!("test-spec-{PASSES}")));
    assert!(
        !shown(&mut vcx, &format!("test-debug-{PASSES}")),
        "a Rust row has no Debug"
    );
    right_click(&mut vcx, &format!("test-spec-{PASSES}"));
    assert!(shown(&mut vcx, "test-menu-run"));
    assert!(
        !shown(&mut vcx, "test-menu-debug"),
        "a Rust row's menu has no Debug"
    );
    click(&mut vcx, "test-menu-run", Modifiers::default());
    wait_until(
        &mut vcx,
        (window, "the one test run"),
        |v: &ExplorerView| v.tests.busy.is_none(),
    );
    assert_eq!(outcome(window, &mut vcx, PASSES), Some(Outcome::Passed));
    assert_eq!(outcome(window, &mut vcx, FAILS), None);
    click(&mut vcx, "test-class-run", Modifiers::default());
    wait_until(&mut vcx, (window, "the module run"), |v: &ExplorerView| {
        v.tests.busy.is_none()
    });
    assert_eq!(
        (
            outcome(window, &mut vcx, FAILS),
            outcome(window, &mut vcx, IGNORED)
        ),
        (Some(Outcome::Failed), Some(Outcome::NotExecuted))
    );
    assert!(
        !root.join("target").exists(),
        "tests build in backback's scratch folder"
    );
    close_all(cx);
}
