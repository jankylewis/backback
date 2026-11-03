use std::path::{Path, PathBuf};

use bb_explorer::Explorer;
use bb_testkit::Outcome;
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext, Window, WindowAppearance};

use crate::ExplorerView;

fn write_fixture(root: &Path) -> PathBuf {
    let project: PathBuf = root.join("Tests.csproj");
    std::fs::write(&project, r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject></PropertyGroup><ItemGroup><PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.1" /><PackageReference Include="NUnit" Version="4.6.1" /><PackageReference Include="NUnit3TestAdapter" Version="5.0.0" /></ItemGroup></Project>"#).unwrap();
    std::fs::write(root.join("OutcomeSpec.cs"), "using NUnit.Framework;\nnamespace Sample;\npublic class OutcomeSpec {\n[Test] public void Passes() => Assert.Pass();\n[Test] public void Fails() => Assert.Fail(\"failure\");\n}\n").unwrap();
    project
}

fn fixture(root: &Path) -> PathBuf {
    let project: PathBuf = write_fixture(root);
    let status: std::process::ExitStatus = std::process::Command::new("dotnet")
        .args(["build", "Tests.csproj", "-c", "Debug", "-nologo", "-v", "q"])
        .current_dir(root)
        .status()
        .unwrap();
    assert!(status.success());
    project
}

fn click(vcx: &mut VisualTestContext, selector: &'static str) {
    let bounds: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds(selector).unwrap();
    vcx.simulate_click(bounds.center(), Modifiers::default());
}

fn wait_for(
    view: &Entity<ExplorerView>,
    vcx: &mut VisualTestContext,
    ready: impl Fn(&ExplorerView) -> bool,
) {
    let started: std::time::Instant = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(30) {
        vcx.run_until_parked();
        if view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| ready(state)) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    panic!("test panel did not settle");
}

/// The project's row has stopped loading.
fn settled(project: &Path) -> impl Fn(&ExplorerView) -> bool + '_ {
    move |state: &ExplorerView| {
        state
            .tests
            .model
            .projects()
            .get(project)
            .is_some_and(|tests: &crate::tests_panel::model::ProjectTests| !tests.loading)
    }
}

#[gpui::test]
fn test_panel_lists_runs_a_class_and_opens_a_failure(cx: &mut TestAppContext) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    let project: PathBuf = fixture(&root);
    let tree: Explorer = Explorer::open(&root).unwrap();
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, (root.clone(), false), WindowAppearance::Dark, cx)
        });
    vcx.run_until_parked();
    click(vcx, "open-tests");
    vcx.run_until_parked();
    click(vcx, "test-project");
    wait_for(&view, vcx, settled(&project));
    view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        let tests: &crate::tests_panel::model::ProjectTests =
            state.tests.model.projects().get(&project).unwrap();
        assert!(
            tests.specs.len() >= 2,
            "error={:?} loading={} projects={:?}",
            tests.error,
            tests.loading,
            state.tests.model.projects().keys().collect::<Vec<_>>()
        );
    });
    click(vcx, "test-class-run");
    wait_for(&view, vcx, |state: &ExplorerView| {
        state.tests.busy.is_none()
    });
    assert_outcomes(&view, vcx, &project);
    click(vcx, "test-spec-Sample.OutcomeSpec.Fails");
    click(vcx, "test-detail");
    vcx.run_until_parked();
    view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        let tab: &crate::editor::Tab = state.tabs.active_tab().unwrap();
        assert_eq!(tab.path, root.join("OutcomeSpec.cs"));
        assert_eq!(tab.document.cursor_line(), 4);
    });
}

fn assert_outcomes(view: &Entity<ExplorerView>, vcx: &mut VisualTestContext, project: &Path) {
    view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        let tests: &crate::tests_panel::model::ProjectTests =
            state.tests.model.projects().get(project).unwrap();
        let outcomes: Vec<Outcome> = tests
            .specs
            .values()
            .filter_map(|result: &Option<bb_testkit::SpecResult>| {
                result
                    .as_ref()
                    .map(|result: &bb_testkit::SpecResult| result.outcome)
            })
            .collect();
        assert!(outcomes.contains(&Outcome::Passed));
        assert!(outcomes.contains(&Outcome::Failed));
    });
}

/// The active tab's file and cursor line.
fn at(view: &Entity<ExplorerView>, vcx: &mut VisualTestContext) -> (PathBuf, usize) {
    view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        let tab: &crate::editor::Tab = state.tabs.active_tab().unwrap();
        (tab.path.clone(), tab.document.cursor_line())
    })
}

/// A window whose Tests panel shows a failure with five stack lines: a
/// frame in `A.cs`, a framework frame, a frame in `B.cs` line 7, a `/_/`
/// frame for `B.cs` line 2, and a file not on this machine.
fn failure_window(
    cx: &mut TestAppContext,
) -> (
    tempfile::TempDir,
    PathBuf,
    Entity<ExplorerView>,
    &mut VisualTestContext,
) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("A.cs"), "class A\n{\n}\n").unwrap();
    std::fs::write(root.join("B.cs"), "line\n".repeat(10)).unwrap();
    let tree: Explorer = Explorer::open(&root).unwrap();
    let start: PathBuf = root.clone();
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, (start, false), WindowAppearance::Dark, cx)
        });
    vcx.run_until_parked();
    click(vcx, "open-tests");
    let stack: String = format!(
        "   at T.S.Fails() in {a}:line 1\n   at System.Runtime.Run()\n   at T.S.Helper() in {b}:line 7\n   at T.S.Mapped() in /_/B.cs:line 2\n   at T.S.Gone() in /nowhere/C.cs:line 3",
        a = root.join("A.cs").display(),
        b = root.join("B.cs").display(),
    );
    view.update(
        vcx,
        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            state.tests.detail = Some(bb_testkit::SpecResult {
                fqn: bb_testkit::Fqn::new("T.S.Fails").unwrap(),
                outcome: Outcome::Failed,
                duration: std::time::Duration::ZERO,
                message: Some(String::from("Expected 3 but was 2")),
                stack: Some(stack),
            });
            cx.notify();
        },
    );
    vcx.run_until_parked();
    (dir, root, view, vcx)
}

/// BBEOD18: every stack frame with a source is a link to its call site;
/// framework frames are plain; `/_/` (deterministic build) paths map to the
/// opened root; a file not on this machine is said in the status bar.
#[gpui::test]
fn each_stack_frame_opens_its_own_call_site(cx: &mut TestAppContext) {
    let (_dir, root, view, vcx): (
        tempfile::TempDir,
        PathBuf,
        Entity<ExplorerView>,
        &mut VisualTestContext,
    ) = failure_window(cx);
    assert!(
        vcx.debug_bounds("stack-link-2").is_none(),
        "a framework frame is plain text"
    );
    click(vcx, "stack-link-3");
    wait_for(&view, vcx, |state: &ExplorerView| {
        state
            .tabs
            .active_tab()
            .is_some_and(|t: &crate::editor::Tab| t.path.ends_with("B.cs"))
    });
    assert_eq!(at(&view, vcx), (root.join("B.cs"), 6));
    click(vcx, "stack-link-4");
    vcx.run_until_parked();
    assert_eq!(at(&view, vcx), (root.join("B.cs"), 1), "the /_/ path");
    click(vcx, "stack-link-5");
    vcx.run_until_parked();
    let status: Option<String> =
        view.read_with(vcx, |s: &ExplorerView, _: &gpui::App| s.status.clone());
    assert_eq!(
        status.as_deref(),
        Some("Cannot open /nowhere/C.cs: not on this machine")
    );
}

#[gpui::test]
fn a_never_built_project_is_built_on_expand_unless_no_write(cx: &mut TestAppContext) {
    for no_write in [true, false] {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let root: PathBuf = dir.path().canonicalize().unwrap();
        let project: PathBuf = write_fixture(&root);
        let tree: Explorer = Explorer::open(&root).unwrap();
        let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
            cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
                crate::launch::fresh_view(
                    tree,
                    (root.clone(), no_write),
                    WindowAppearance::Dark,
                    cx,
                )
            });
        vcx.run_until_parked();
        click(vcx, "open-tests");
        vcx.run_until_parked();
        click(vcx, "test-project");
        wait_for(&view, vcx, settled(&project));
        let (specs, error): (usize, Option<String>) =
            view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
                let tests: &crate::tests_panel::model::ProjectTests =
                    state.tests.model.projects().get(&project).unwrap();
                (tests.specs.len(), tests.error.clone())
            });
        if no_write {
            assert!(
                error
                    .as_deref()
                    .is_some_and(|text: &str| text.contains("--no-write")),
                "{error:?}"
            );
            assert!(!root.join("bin").exists(), "--no-write builds nothing");
        } else {
            assert_eq!(error, None);
            assert!(specs >= 2, "{specs} specs listed after the build");
            assert!(root.join("bin").exists(), "the project was built");
        }
    }
}
