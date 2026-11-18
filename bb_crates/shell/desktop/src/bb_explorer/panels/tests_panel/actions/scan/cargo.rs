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
        bb_cargo::members(root, METADATA_DEADLINE).map_err(|e: CargoError| e.to_string())?;
    Ok(found
        .members
        .into_iter()
        .map(|m: Member| m.manifest)
        .collect())
}

/// The projects to show: the .NET test projects when `csharp`, then the
/// Cargo members when `rust`, and the run settings found.
pub(crate) fn scan_all(
    root: &Path,
    (csharp, rust): (bool, bool),
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let (mut projects, settings): (Vec<PathBuf>, Vec<PathBuf>) = if csharp {
        super::scan(root)?
    } else {
        (Vec::new(), Vec::new())
    };
    if rust {
        projects.extend(members(root)?);
    }
    Ok((projects, settings))
}

/// Where `manifest`'s tests run: its package, the workspace root, and the
/// window's scratch test folder.
fn target(root: &Path, manifest: &Path) -> Result<TestTarget, String> {
    let found: CargoWorkspace =
        bb_cargo::members(root, METADATA_DEADLINE).map_err(|e: CargoError| e.to_string())?;
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
/// (BBEOD26), Cargo with Rust active and a `Cargo.toml` (BBEOD27); with
/// neither the panel says why.
pub(crate) fn discovery(view: &mut ExplorerView) -> (bool, bool) {
    let csharp: bool = view.langs.start(bb_lang::Tool::TestDiscovery);
    let rust: bool = crate::intel::bb_worker::find_cargo(&view.root).is_some()
        && view.langs.start(bb_lang::Tool::Cargo);
    if !csharp && !rust {
        view.tests.message = Some(String::from("Tests: neither C# nor Rust is active here"));
    }
    (csharp, rust)
}

/// Whether `path`'s tests may be listed: its language is active.
pub(crate) fn may_list(view: &mut ExplorerView, path: &std::path::Path) -> bool {
    let tool: bb_lang::Tool = match crate::tests_panel::model::ProjectKind::of(path) {
        crate::tests_panel::model::ProjectKind::Dotnet => bb_lang::Tool::TestDiscovery,
        crate::tests_panel::model::ProjectKind::Cargo => bb_lang::Tool::Cargo,
    };
    view.langs.start(tool)
}
