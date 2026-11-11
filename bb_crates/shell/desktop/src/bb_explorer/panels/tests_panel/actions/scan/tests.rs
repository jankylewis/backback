//! The filter's search: the word it reads the sources for, which projects
//! hold it, and in a real window (`--no-write`, so nothing is built) that
//! only the project whose sources hold a test's name is listed.

use std::path::{Path, PathBuf};

use bb_explorer::Explorer;
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext, Window, WindowAppearance};

use super::find::{holding, word};
use crate::tests_panel::model::ProjectTests;
use crate::ExplorerView;

const PROJECT: &str = r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject></PropertyGroup><ItemGroup><PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.1" /><PackageReference Include="NUnit" Version="4.6.1" /><PackageReference Include="NUnit3TestAdapter" Version="5.0.0" /></ItemGroup></Project>"#;

/// `<root>/<folder>/<name>.csproj` with one spec file whose test is
/// `Mst_In<name>`; the project file's path.
fn project(root: &Path, folder: &str, name: &str) -> PathBuf {
    let dir: PathBuf = root.join(folder);
    std::fs::create_dir_all(&dir).unwrap();
    let source: String = format!(
        "using NUnit.Framework;\nnamespace Sample;\npublic class {name}Spec {{\n[Test] public void Mst_In{name}() => Assert.Pass();\n}}\n"
    );
    std::fs::write(dir.join(format!("{name}Spec.cs")), source).unwrap();
    let file: PathBuf = dir.join(format!("{name}.csproj"));
    std::fs::write(&file, PROJECT).unwrap();
    file
}

#[test]
fn a_filter_is_searched_by_its_longest_word() {
    assert_eq!(word("Mst_DownloadIms"), Some("Mst_DownloadIms"));
    assert_eq!(
        word("Sample.Auth.OutcomeSpec.Counts(1)"),
        Some("OutcomeSpec")
    );
    assert_eq!(word("  a.bc "), None, "under three letters is everywhere");
    assert_eq!(word(""), None);
}

#[test]
fn only_projects_whose_own_sources_hold_the_word_are_found() {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    let alpha: PathBuf = project(&root, "Alpha", "Alpha");
    let inner: PathBuf = project(&root, "Alpha/Inner", "Inner");
    let beta: PathBuf = project(&root, "Beta", "Beta");
    std::fs::create_dir_all(root.join("Beta/obj")).unwrap();
    std::fs::write(root.join("Beta/obj/Gen.cs"), "Mst_InAlpha").unwrap();
    std::fs::write(root.join("Beta/notes.txt"), "Mst_InAlpha").unwrap();
    let all: Vec<PathBuf> = vec![alpha.clone(), inner.clone(), beta.clone()];
    assert_eq!(
        holding(&all, &all, "MST_INALPHA"),
        std::slice::from_ref(&alpha)
    );
    assert_eq!(
        holding(&all, &all, "Mst_InInner"),
        std::slice::from_ref(&inner),
        "a project inside Alpha's folder keeps its own files"
    );
    assert_eq!(
        holding(&[beta], &all, "NUnit"),
        [root.join("Beta/Beta.csproj")]
    );
    assert_eq!(holding(&all, &all, "Mst_Nowhere"), Vec::<PathBuf>::new());
}

fn click(vcx: &mut VisualTestContext, selector: &'static str) {
    let bounds: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds(selector).unwrap();
    vcx.simulate_click(bounds.center(), Modifiers::default());
    vcx.run_until_parked();
}

/// (open, the list's error, listed tests) of `project`, once nothing loads.
fn settled(
    view: &Entity<ExplorerView>,
    vcx: &mut VisualTestContext,
    project: &Path,
) -> (bool, Option<String>, usize) {
    let started: std::time::Instant = std::time::Instant::now();
    loop {
        vcx.run_until_parked();
        let quiet: bool = view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
            let loading: bool = state
                .tests
                .model
                .projects()
                .values()
                .any(|tests: &ProjectTests| tests.loading);
            !loading && crate::tests_panel::find::hint(state).is_none()
        });
        if quiet {
            break;
        }
        assert!(started.elapsed().as_secs() < 60, "the search did not end");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        let tests: &ProjectTests = state.tests.model.projects().get(project).unwrap();
        (tests.expanded, tests.error.clone(), tests.specs.len())
    })
}

type Two<'a> = (
    tempfile::TempDir,
    [PathBuf; 2],
    Entity<ExplorerView>,
    &'a mut VisualTestContext,
);

/// A `--no-write` window (nothing is built) on the Tests side over two
/// projects, Alpha and Beta, neither listed.
fn two_projects(cx: &mut TestAppContext) -> Two<'_> {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    let alpha: PathBuf = project(&root, "Alpha", "Alpha");
    let beta: PathBuf = project(&root, "Beta", "Beta");
    let tree: Explorer = Explorer::open(&root).unwrap();
    let (view, vcx): (Entity<ExplorerView>, &mut VisualTestContext) =
        cx.add_window_view(|_: &mut Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::launch::fresh_view(tree, (root, true), WindowAppearance::Dark, cx)
        });
    vcx.run_until_parked();
    click(vcx, "open-tests");
    (dir, [alpha, beta], view, vcx)
}

#[gpui::test]
fn enter_in_the_filter_lists_only_the_project_whose_sources_hold_the_name(cx: &mut TestAppContext) {
    let (_dir, [alpha, beta], view, vcx): Two<'_> = two_projects(cx);
    click(vcx, "test-filter");
    assert!(
        vcx.debug_bounds("test-filter-hint").is_none(),
        "empty filter"
    );
    vcx.simulate_keystrokes("i n b e t a");
    assert!(
        vcx.debug_bounds("test-filter-hint").is_some(),
        "Enter searches"
    );
    let typed: bool = view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        state
            .tests
            .model
            .projects()
            .values()
            .all(|tests: &ProjectTests| !tests.expanded)
    });
    assert!(typed, "typing lists nothing");
    vcx.simulate_keystrokes("enter");
    let (open, error, _): (bool, Option<String>, usize) = settled(&view, vcx, &beta);
    assert!(open, "the project that holds Mst_InBeta opened");
    assert!(
        error
            .as_deref()
            .is_some_and(|text: &str| text.contains("--no-write")),
        "it was listed (and --no-write refused the build): {error:?}"
    );
    assert_eq!(
        settled(&view, vcx, &alpha),
        (false, None, 0),
        "Alpha untouched"
    );
    click(vcx, "test-filter");
    vcx.simulate_keystrokes("cmd-backspace");
    vcx.write_to_clipboard(gpui::ClipboardItem::new_string("Mst_Nowhere".to_owned()));
    vcx.simulate_keystrokes("cmd-v");
    assert_eq!(settled(&view, vcx, &alpha), (false, None, 0));
    let message: Option<String> = view.read_with(vcx, |state: &ExplorerView, _: &gpui::App| {
        state.tests.message.clone()
    });
    assert_eq!(
        message.as_deref(),
        Some("No test source holds \"Mst_Nowhere\"")
    );
}
