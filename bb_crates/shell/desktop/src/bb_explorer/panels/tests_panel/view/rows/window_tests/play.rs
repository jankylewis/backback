//! BB-BUG-004: a click on ▶ in one test's row, in a real window on real
//! dotnet, runs that test only and its row shows the verdict.

use std::path::PathBuf;

use bb_explorer::Explorer;
use bb_testkit::{Fqn, Outcome, SpecResult};
use gpui::{Entity, TestAppContext, VisualTestContext, Window, WindowAppearance};

use super::super::super::super::window_tests::{fixture, settled, wait_for};
use super::click;
use crate::ExplorerView;

const FAILS: &str = "Sample.OutcomeSpec.Fails";
const PASSES: &str = "Sample.OutcomeSpec.Passes";

fn outcome(state: &ExplorerView, project: &PathBuf, name: &str) -> Option<Outcome> {
    let spec: Fqn = Fqn::new(name).unwrap();
    state
        .tests
        .model
        .projects()
        .get(project)
        .and_then(|tests: &crate::tests_panel::model::ProjectTests| tests.specs.get(&spec))
        .and_then(Option::as_ref)
        .map(|result: &SpecResult| result.outcome)
}

#[gpui::test]
fn play_on_one_test_runs_only_that_test_and_shows_its_verdict(cx: &mut TestAppContext) {
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
    click(vcx, "test-project");
    wait_for(&view, vcx, settled(&project));
    click(vcx, "test-ns-Sample");
    click(vcx, "test-class-Sample.OutcomeSpec");
    click(vcx, "test-play-Sample.OutcomeSpec.Fails");
    let ran: PathBuf = project.clone();
    wait_for(&view, vcx, move |state: &ExplorerView| {
        state.tests.busy.is_none() && outcome(state, &ran, FAILS).is_some()
    });
    view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        assert_eq!(outcome(state, &project, FAILS), Some(Outcome::Failed));
        assert_eq!(
            outcome(state, &project, PASSES),
            None,
            "only the clicked test ran"
        );
    });
}
