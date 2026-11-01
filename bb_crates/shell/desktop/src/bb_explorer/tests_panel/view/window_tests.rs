use std::path::{Path, PathBuf};

use bb_explorer::Explorer;
use bb_testkit::Outcome;
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext, Window, WindowAppearance};

use crate::ExplorerView;

fn fixture(root: &Path) -> PathBuf {
    let project: PathBuf = root.join("Tests.csproj");
    std::fs::write(&project, r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject></PropertyGroup><ItemGroup><PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.1" /><PackageReference Include="NUnit" Version="4.6.1" /><PackageReference Include="NUnit3TestAdapter" Version="5.0.0" /></ItemGroup></Project>"#).unwrap();
    std::fs::write(root.join("OutcomeSpec.cs"), "using NUnit.Framework;\nnamespace Sample;\npublic class OutcomeSpec {\n[Test] public void Passes() => Assert.Pass();\n[Test] public void Fails() => Assert.Fail(\"failure\");\n}\n").unwrap();
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
    wait_for(&view, vcx, |state: &ExplorerView| {
        state
            .tests
            .model
            .projects()
            .get(&project)
            .is_some_and(|tests: &crate::tests_panel::model::ProjectTests| !tests.loading)
    });
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
