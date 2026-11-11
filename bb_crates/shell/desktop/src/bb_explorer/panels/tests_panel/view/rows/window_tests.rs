//! The Tests panel's tree, with no dotnet: specs are merged straight into
//! the model. BB-BUG-024 (fold), BB-BUG-025 (open), BB-BUG-026 (stop), and
//! the filter's paste and search over a listed project.

use std::path::{Path, PathBuf};

use bb_explorer::Explorer;
use bb_testkit::{Fqn, SpecList, StopFlag};
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext, Window, WindowAppearance};

use crate::ExplorerView;

const SPEC: &str = "using NUnit.Framework;\nnamespace Sample.Auth;\npublic class OutcomeSpec\n{\n    [Test] public void Passes() => Assert.Pass();\n    [Test] public void Fails() => Assert.Fail();\n    [TestCase(1)][TestCase(2)] public void Counts(int n) { }\n}\n";
const NAMES: [&str; 4] = [
    "Sample.Auth.OutcomeSpec.Passes",
    "Sample.Auth.OutcomeSpec.Fails",
    "Sample.Auth.OutcomeSpec.Counts(1)",
    "Sample.Auth.OutcomeSpec.Counts(2)",
];

type Rig<'a> = (
    tempfile::TempDir,
    PathBuf,
    Entity<ExplorerView>,
    &'a mut VisualTestContext,
);

/// A window on the Tests side with one open project of four specs.
fn tree_window(cx: &mut TestAppContext) -> Rig<'_> {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("OutcomeSpec.cs"), SPEC).unwrap();
    let project: PathBuf = root.join("Tests.csproj");
    let tree: Explorer = Explorer::open(&root).unwrap();
    let start: PathBuf = root.clone();
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, (start, false), WindowAppearance::Dark, cx)
        });
    vcx.run_until_parked();
    let listed: PathBuf = project.clone();
    view.update(
        vcx,
        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            state.tests.side = crate::tests_panel::Side::Tests;
            state.tests.scan = crate::tests_panel::Scan::Done;
            state.tests.model.merge(&listed, listing(&NAMES));
            if let Some(tests) = state.tests.model.project_mut(&listed) {
                tests.expanded = true;
            }
            cx.notify();
        },
    );
    vcx.run_until_parked();
    (dir, project, view, vcx)
}

fn listing(names: &[&str]) -> SpecList {
    SpecList {
        project: String::from("Tests"),
        assembly: PathBuf::from("Tests.dll"),
        specs: names
            .iter()
            .map(|name: &&str| Fqn::new(name).unwrap())
            .collect(),
        elapsed: std::time::Duration::ZERO,
    }
}

fn click(vcx: &mut VisualTestContext, selector: &'static str) {
    let bounds: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds(selector).unwrap();
    vcx.simulate_click(bounds.center(), Modifiers::default());
    vcx.run_until_parked();
}

fn shown(vcx: &mut VisualTestContext, selector: &'static str) -> bool {
    vcx.debug_bounds(selector).is_some()
}

#[gpui::test]
fn namespace_class_and_method_heads_fold_and_open(cx: &mut TestAppContext) {
    let (_dir, _project, _view, vcx): Rig<'_> = tree_window(cx);
    assert!(shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"));
    click(vcx, "test-class-Sample.Auth.OutcomeSpec");
    assert!(
        !shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"),
        "class folded"
    );
    assert!(!shown(vcx, "test-method-Sample.Auth.OutcomeSpec.Counts"));
    assert!(shown(vcx, "test-ns-Sample.Auth"));
    click(vcx, "test-class-Sample.Auth.OutcomeSpec");
    assert!(
        shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"),
        "class open"
    );
    click(vcx, "test-ns-Sample.Auth");
    assert!(
        !shown(vcx, "test-class-Sample.Auth.OutcomeSpec"),
        "namespace folded"
    );
    assert!(!shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Fails"));
    click(vcx, "test-ns-Sample.Auth");
    assert!(
        shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Fails"),
        "namespace open"
    );
    click(vcx, "test-method-Sample.Auth.OutcomeSpec.Counts");
    assert!(
        !shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Counts(1)"),
        "method folded"
    );
    assert!(shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"));
}

#[gpui::test]
fn a_click_on_a_spec_opens_its_method(cx: &mut TestAppContext) {
    let (_dir, project, view, vcx): Rig<'_> = tree_window(cx);
    let root: PathBuf = project.parent().map(Path::to_path_buf).unwrap();
    click(vcx, "test-spec-Sample.Auth.OutcomeSpec.Fails");
    let started: std::time::Instant = std::time::Instant::now();
    let mut place: Option<(PathBuf, usize)> = None;
    while started.elapsed() < std::time::Duration::from_secs(10) && place.is_none() {
        vcx.run_until_parked();
        place = view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
            state
                .tabs
                .active_tab()
                .map(|tab: &crate::editor::Tab| (tab.path.clone(), tab.document.cursor_line()))
        });
    }
    assert_eq!(place, Some((root.join("OutcomeSpec.cs"), 5)));
}

#[gpui::test]
fn a_running_spec_and_the_toolbar_show_a_stop_that_stops_the_run(cx: &mut TestAppContext) {
    let (_dir, project, view, vcx): Rig<'_> = tree_window(cx);
    assert!(!shown(vcx, "tests-stop"), "no stop while idle");
    let flag: StopFlag = StopFlag::new();
    let held: StopFlag = flag.clone();
    view.update(
        vcx,
        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            state.tests.busy = Some(project.clone());
            state.tests.stop = Some(held);
            state.tests.running_specs.insert(
                project.clone(),
                [Fqn::new("Sample.Auth.OutcomeSpec.Passes").unwrap()]
                    .into_iter()
                    .collect(),
            );
            cx.notify();
        },
    );
    vcx.run_until_parked();
    assert!(shown(vcx, "tests-stop"));
    assert!(
        !shown(vcx, "test-stop-Sample.Auth.OutcomeSpec.Fails"),
        "Fails is not running"
    );
    click(vcx, "test-stop-Sample.Auth.OutcomeSpec.Passes");
    assert!(flag.stopped(), "the row's stop set the run's stop flag");
}

fn filter_text(view: &Entity<ExplorerView>, vcx: &mut VisualTestContext) -> String {
    view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        state.tests.filter.clone()
    })
}

/// A test's name copied in the editor and pasted in the filter: the listed
/// project that holds it opens and shows only that test, in any letter case.
#[gpui::test]
fn a_pasted_name_opens_the_listed_project_on_only_that_test(cx: &mut TestAppContext) {
    let (_dir, project, view, vcx): Rig<'_> = tree_window(cx);
    view.update(
        vcx,
        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            state.tests.model.project_mut(&project).unwrap().expanded = false;
            cx.notify();
        },
    );
    vcx.run_until_parked();
    assert!(!shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Fails"));
    click(vcx, "test-filter");
    vcx.write_to_clipboard(gpui::ClipboardItem::new_string(
        "  outcomespec.FAILS \nsecond line".to_owned(),
    ));
    vcx.simulate_keystrokes("cmd-v");
    vcx.run_until_parked();
    assert_eq!(filter_text(&view, vcx), "outcomespec.FAILS");
    assert!(shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Fails"));
    assert!(!shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"));
    vcx.simulate_keystrokes("cmd-backspace");
    assert_eq!(filter_text(&view, vcx), "", "Cmd+Backspace clears it");
    assert!(shown(vcx, "test-spec-Sample.Auth.OutcomeSpec.Passes"));
}

fn status_of(view: &Entity<ExplorerView>, vcx: &mut VisualTestContext) -> Option<String> {
    view.read_with(vcx, |s: &ExplorerView, _: &gpui::App| s.status.clone())
}

/// Headmate 2026-11-11: a right-click on a test opens Run and Debug; the
/// row's Debug and the menu's Debug both start a debug session (refused
/// here by --no-write, so no build runs); Run runs the test.
#[gpui::test]
fn a_right_click_on_a_test_offers_run_and_debug(cx: &mut TestAppContext) {
    let (_dir, project, view, vcx): Rig<'_> = tree_window(cx);
    view.update(
        vcx,
        |state: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            state.tabs.set_no_write();
        },
    );
    let row: gpui::Bounds<gpui::Pixels> = vcx
        .debug_bounds("test-spec-Sample.Auth.OutcomeSpec.Fails")
        .unwrap();
    vcx.simulate_mouse_down(row.center(), gpui::MouseButton::Right, Modifiers::default());
    vcx.run_until_parked();
    assert!(shown(vcx, "test-menu-run") && shown(vcx, "test-menu-debug"));
    click(vcx, "test-menu-debug");
    assert!(
        status_of(&view, vcx).is_some_and(|s: String| s.starts_with("Debug refused")),
        "the menu's Debug starts a debug session"
    );
    assert!(!shown(vcx, "test-menu"), "picking an item closes the menu");
    view.update(
        vcx,
        |state: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            state.status = None;
        },
    );
    click(vcx, "test-debug-Sample.Auth.OutcomeSpec.Passes");
    assert!(status_of(&view, vcx).is_some_and(|s: String| s.starts_with("Debug refused")));
    vcx.simulate_mouse_down(row.center(), gpui::MouseButton::Right, Modifiers::default());
    vcx.run_until_parked();
    view.update(
        vcx,
        |state: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            state.tests.message = None;
        },
    );
    click(vcx, "test-menu-run");
    // The fixture has no real project: the run ends at once with its error.
    let said: Option<String> = view.read_with(vcx, |s: &ExplorerView, _: &gpui::App| {
        s.tests.message.clone()
    });
    assert!(said.is_some(), "Run ran the test of {}", project.display());
}
