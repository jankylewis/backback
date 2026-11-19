//! The Tests panel for Rust (BBEOD27): the Cargo workspace's members are
//! its projects; `cargo test -- --list` lists one, `cargo test` runs it.
//! Both build in backback's scratch folder, never the project's `target/`.

use std::path::{Path, PathBuf};

use bb_cargo::run::cargo_test::{self, TestTarget};
use bb_cargo::{CargoError, CargoWorkspace, Member, METADATA_DEADLINE};
use bb_testkit::{RunReport, Selection, SpecList, StopFlag};

use crate::ExplorerView;

/// The manifests of every member of the workspace at `root`.
pub(crate) fn members(root: &Path) -> Result<Vec<PathBuf>, String> {
    let found: CargoWorkspace =
        bb_cargo::members(root, METADATA_DEADLINE).map_err(|e: CargoError| e.summary())?;
    Ok(found
        .members
        .into_iter()
        .map(|m: Member| m.manifest)
        .collect())
}

/// Which languages' tests the panel lists (BBEOD26/27/28).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct Kinds {
    pub(crate) csharp: bool,
    pub(crate) rust: bool,
    pub(crate) python: bool,
}

/// The projects to show: the .NET test projects with `csharp`, the Cargo
/// members with `rust`, the pytest root with `python`, and the run
/// settings found.
pub(crate) fn scan_all(root: &Path, kinds: Kinds) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let (mut projects, settings): (Vec<PathBuf>, Vec<PathBuf>) = if kinds.csharp {
        super::scan(root)?
    } else {
        (Vec::new(), Vec::new())
    };
    if kinds.rust {
        projects.extend(members(root)?);
    }
    if kinds.python {
        projects.extend(super::python::root_file(root));
    }
    Ok((projects, settings))
}

/// Where `manifest`'s tests run: its package, the workspace root, and the
/// window's scratch test folder.
fn target(root: &Path, manifest: &Path) -> Result<TestTarget, String> {
    let found: CargoWorkspace =
        bb_cargo::members(root, METADATA_DEADLINE).map_err(|e: CargoError| e.summary())?;
    let member: Member = found
        .members
        .into_iter()
        .find(|m: &Member| m.manifest == manifest)
        .ok_or_else(|| format!("not a workspace member: {}", manifest.display()))?;
    Ok(TestTarget {
        root: found.root,
        package: member.name,
        target_dir: crate::intel::bb_worker::scratch(root, "tests"),
    })
}

/// List `manifest`'s tests.
pub(crate) fn list(root: &Path, manifest: &Path) -> Result<SpecList, String> {
    let at: TestTarget = target(root, manifest)?;
    cargo_test::list(&at, manifest).map_err(|e: CargoError| e.to_string())
}

/// Run `selection` of `manifest`'s tests.
pub(crate) fn run(
    root: &Path,
    manifest: &Path,
    selection: &Selection,
    stop: &StopFlag,
) -> Result<RunReport, String> {
    let at: TestTarget = target(root, manifest)?;
    cargo_test::run(&at, selection, stop).map_err(|e: CargoError| e.to_string())
}

/// Which languages' tests may be discovered: .NET with C# active
/// (BBEOD26), Cargo with Rust active and a `Cargo.toml` (BBEOD27), pytest
/// with Python active and a pytest root file (BBEOD28); with none the
/// panel says why.
pub(crate) fn discovery(view: &mut ExplorerView) -> Kinds {
    let kinds: Kinds = Kinds {
        csharp: view.langs.start(bb_lang::Tool::TestDiscovery),
        rust: crate::intel::bb_worker::find_cargo(&view.root).is_some()
            && view.langs.start(bb_lang::Tool::Cargo),
        python: super::python::root_file(&view.root).is_some()
            && view.langs.start(bb_lang::Tool::Pytest),
    };
    if kinds == Kinds::default() {
        view.tests.message = Some(String::from(
            "Tests: no active C#, Rust or Python (pytest root) here",
        ));
    }
    kinds
}

/// Whether `path`'s tests may be listed: its language is active.
pub(crate) fn may_list(view: &mut ExplorerView, path: &std::path::Path) -> bool {
    let tool: bb_lang::Tool = match crate::tests_panel::model::ProjectKind::of(path) {
        crate::tests_panel::model::ProjectKind::Dotnet => bb_lang::Tool::TestDiscovery,
        crate::tests_panel::model::ProjectKind::Cargo => bb_lang::Tool::Cargo,
        crate::tests_panel::model::ProjectKind::Pytest => bb_lang::Tool::Pytest,
    };
    view.langs.start(tool)
}
