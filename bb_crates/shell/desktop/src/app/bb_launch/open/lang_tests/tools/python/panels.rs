//! BBEOD28 panels on this machine's tools, by mouse and keys: Cmd+Shift+B in
//! a Python window checks the folder (pyright, or the interpreter's syntax
//! check while pyright is missing) and a click on the error opens its line;
//! the Tests panel lists the pytest root and either runs pass, fail and
//! skip with no Debug on a Python row, or shows pytest's install command.
//! Nothing is written into the project either way.

use std::path::{Path, PathBuf};

use bb_lang::{Lang, LangSet, Tool};
use bb_python::fixtures::{self, Shape, BROKEN_FILE, BROKEN_LINE};
use bb_python::run::bb_pytest::missing_hint;
use bb_testkit::Outcome;
use gpui::{App, Modifiers, TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::{close_all, open_with};
use super::super::rust::panel_kit::{click, outcome, right_click, shown, wait_until};
use crate::ExplorerView;

const PASSES: &str = "tests.test_cart.test_passes";
const FAILS: &str = "tests.test_cart.test_fails";
const SKIPPED: &str = "tests.test_cart.test_skipped";

type Opened = (
    tempfile::TempDir,
    PathBuf,
    WindowHandle<ExplorerView>,
    VisualTestContext,
);

fn python_window(cx: &mut TestAppContext, shape: Shape) -> Opened {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = base.path().canonicalize().unwrap().join("py");
    fixtures::write(&root, shape).unwrap();
    let window: WindowHandle<ExplorerView> = open_with(cx, &root, LangSet::of(&[Lang::Python]));
    let vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    (base, root, window, vcx)
}

fn listing(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut todo: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = todo.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path: PathBuf = entry.unwrap().path();
            if path.is_dir() {
                todo.push(path.clone());
            }
            found.push(path);
        }
    }
    found.sort();
    found
}

fn error_place(
    window: WindowHandle<ExplorerView>,
    vcx: &VisualTestContext,
) -> (String, Option<usize>) {
    window
        .read_with(vcx, |v: &ExplorerView, _: &App| {
            let panel: &crate::intel::model::popups::PlacesPanel = v.intel.references.as_ref().unwrap();
            let at: Option<usize> = panel.places.iter().position(|p: &crate::intel::bb_worker::protocol::Place| {
                p.preview.starts_with("error")
                    && matches!(&p.location.uri, bb_intel::DocUri::File(f) if f.as_path().ends_with(BROKEN_FILE))
            });
            (panel.title.clone(), at)
        })
        .unwrap()
}

#[gpui::test]
fn cmd_shift_b_in_a_python_window_lists_the_syntax_error_and_a_click_opens_its_line(
    cx: &mut TestAppContext,
) {
    let (_base, root, window, mut vcx): Opened = python_window(cx, Shape::Broken);
    let before: Vec<PathBuf> = listing(&root);
    let cart: PathBuf = root.join(fixtures::CART_FILE);
    window
        .update(
            &mut vcx,
            |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
                v.open_file(&cart, w, cx);
            },
        )
        .unwrap();
    click(&mut vcx, "editor-area", Modifiers::default());
    vcx.update(|_: &mut gpui::Window, cx: &mut App| crate::launch::bind_keys(cx));
    vcx.simulate_keystrokes("cmd-shift-b");
    wait_until(
        &mut vcx,
        (window, "the python check"),
        |v: &ExplorerView| {
            v.status
                .as_deref()
                .is_some_and(|s: &str| s.starts_with("Build "))
        },
    );
    let (title, error_at): (String, Option<usize>) = error_place(window, &vcx);
    assert!(title.starts_with("Build failed: "), "{title}");
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
            root.join(BROKEN_FILE),
            usize::try_from(BROKEN_LINE - 1).unwrap()
        )
    );
    assert_eq!(listing(&root), before);
    close_all(cx);
}

fn run_every_test(vcx: &mut VisualTestContext, window: WindowHandle<ExplorerView>) {
    click(vcx, "test-ns-tests", Modifiers::default());
    click(vcx, "test-class-tests.test_cart", Modifiers::default());
    assert!(shown(vcx, &format!("test-spec-{PASSES}")));
    assert!(
        !shown(vcx, &format!("test-debug-{PASSES}")),
        "a Python row has no Debug"
    );
    right_click(vcx, &format!("test-spec-{PASSES}"));
    assert!(shown(vcx, "test-menu-run"));
    assert!(
        !shown(vcx, "test-menu-debug"),
        "a Python row's menu has no Debug"
    );
    click(vcx, "test-menu-run", Modifiers::default());
    wait_until(vcx, (window, "the one test run"), |v: &ExplorerView| {
        v.tests.busy.is_none()
    });
    assert_eq!(outcome(window, vcx, PASSES), Some(Outcome::Passed));
    click(vcx, "test-class-run", Modifiers::default());
    wait_until(vcx, (window, "the module run"), |v: &ExplorerView| {
        v.tests.busy.is_none()
    });
    assert_eq!(
        (outcome(window, vcx, FAILS), outcome(window, vcx, SKIPPED)),
        (Some(Outcome::Failed), Some(Outcome::NotExecuted))
    );
}

#[gpui::test]
fn the_tests_panel_runs_pytest_with_no_debug_or_shows_its_install_command(cx: &mut TestAppContext) {
    let (_base, root, window, mut vcx): Opened = python_window(cx, Shape::Clean);
    let before: Vec<PathBuf> = listing(&root);
    click(&mut vcx, "open-tests", Modifiers::default());
    wait_until(&mut vcx, (window, "the test scan"), |v: &ExplorerView| {
        v.tests.scan == crate::tests_panel::Scan::Done
    });
    let started: Vec<Tool> = window
        .read_with(&vcx, |v: &ExplorerView, _: &App| v.langs.started.clone())
        .unwrap();
    assert_eq!(started, vec![Tool::Pyright, Tool::Pytest]);
    click(&mut vcx, "test-project-py", Modifiers::default());
    wait_until(&mut vcx, (window, "the pytest list"), |v: &ExplorerView| {
        v.tests
            .model
            .projects()
            .values()
            .any(|p: &crate::tests_panel::model::ProjectTests| {
                !p.loading && (p.specs.len() == 3 || p.error.is_some())
            })
    });
    let error: Option<String> = window
        .read_with(&vcx, |v: &ExplorerView, _: &App| {
            v.tests
                .model
                .projects()
                .values()
                .find_map(|p: &crate::tests_panel::model::ProjectTests| p.error.clone())
        })
        .unwrap();
    let python: PathBuf = crate::intel::bb_worker::interpreter(&root);
    match error {
        None => run_every_test(&mut vcx, window),
        Some(said) => {
            assert!(
                std::env::var_os("BACKBACK_REQUIRE_TOOLS").is_none(),
                "BACKBACK_REQUIRE_TOOLS is set: {said}"
            );
            assert_eq!(said, missing_hint(&python));
            assert!(shown(&mut vcx, "test-project-error-py"));
        }
    }
    assert_eq!(listing(&root), before);
    close_all(cx);
}
