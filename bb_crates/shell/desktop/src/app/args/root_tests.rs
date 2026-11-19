//! BBEOD33 item 1: the folder a file argument opens.

use std::path::{Path, PathBuf};

use super::bb_root::{project_root, walk};

fn tree(files: &[&str]) -> (tempfile::TempDir, PathBuf) {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().to_path_buf();
    for file in files {
        let path: PathBuf = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "x\n").unwrap();
    }
    (dir, root)
}

fn no_cargo(_: &Path) -> Option<PathBuf> {
    None
}

#[test]
fn a_rust_file_roots_at_the_workspace_cargo_names() {
    let (_dir, root): (tempfile::TempDir, PathBuf) = tree(&[
        "Cargo.toml",
        "crates/alpha/Cargo.toml",
        "crates/alpha/src/lib.rs",
    ]);
    let file: PathBuf = root.join("crates/alpha/src/lib.rs");
    let workspace: PathBuf = root.clone();
    let cargo: &dyn Fn(&Path) -> Option<PathBuf> =
        &move |_: &Path| -> Option<PathBuf> { Some(workspace.canonicalize().unwrap()) };
    assert_eq!(project_root(&file, None, cargo), Some(root.clone()));
    assert_eq!(
        project_root(&file, None, &no_cargo),
        Some(root.join("crates/alpha")),
        "cargo gives no answer (a broken Cargo.toml): the nearest Cargo.toml"
    );
}

#[test]
fn a_cargo_root_outside_the_walk_is_not_taken() {
    let (_dir, root): (tempfile::TempDir, PathBuf) =
        tree(&["app/.git/HEAD", "app/Cargo.toml", "app/src/main.rs"]);
    let file: PathBuf = root.join("app/src/main.rs");
    let above: PathBuf = root.clone();
    let cargo: &dyn Fn(&Path) -> Option<PathBuf> =
        &move |_: &Path| -> Option<PathBuf> { Some(above.clone()) };
    assert_eq!(project_root(&file, None, cargo), Some(root.join("app")));
}

#[test]
fn a_python_file_roots_at_the_nearest_pyproject_or_setup_py() {
    let (_dir, root): (tempfile::TempDir, PathBuf) = tree(&[
        "pyproject.toml",
        "pkg/setup.py",
        "pkg/mod/a.py",
        "tool/b.py",
    ]);
    assert_eq!(
        project_root(&root.join("pkg/mod/a.py"), None, &no_cargo),
        Some(root.join("pkg"))
    );
    assert_eq!(
        project_root(&root.join("tool/b.py"), None, &no_cargo),
        Some(root.clone())
    );
}

#[test]
fn a_mixed_workspace_roots_at_the_outermost_root_holding_the_files_project() {
    let (_dir, root): (tempfile::TempDir, PathBuf) = tree(&[
        "App.sln",
        "tools/rs/Cargo.toml",
        "tools/rs/src/main.rs",
        "tools/py/pyproject.toml",
        "tools/py/run.py",
    ]);
    for file in ["tools/rs/src/main.rs", "tools/py/run.py"] {
        assert_eq!(
            project_root(&root.join(file), None, &no_cargo),
            Some(root.clone()),
            "{file}"
        );
    }
}

#[test]
fn a_file_with_no_project_of_its_language_keeps_its_own_folder() {
    let (_dir, root): (tempfile::TempDir, PathBuf) = tree(&["Cargo.toml", "scripts/x.py"]);
    assert_eq!(
        project_root(&root.join("scripts/x.py"), None, &no_cargo),
        None
    );
}

#[test]
fn the_walk_stops_at_a_git_root_and_never_reaches_home() {
    let (_dir, root): (tempfile::TempDir, PathBuf) =
        tree(&["Cargo.toml", "repo/.git/HEAD", "repo/src/main.rs"]);
    let folder: PathBuf = root.join("repo/src");
    assert_eq!(walk(&folder, None), vec![folder.clone(), root.join("repo")]);
    assert_eq!(
        project_root(&root.join("repo/src/main.rs"), None, &no_cargo),
        None,
        "the Cargo.toml above the git root is outside the walk"
    );
    let (_home_dir, home): (tempfile::TempDir, PathBuf) = tree(&["Cargo.toml", "code/src/main.rs"]);
    assert_eq!(
        walk(&home.join("code/src"), Some(&home)),
        vec![home.join("code/src"), home.join("code")]
    );
    assert_eq!(
        project_root(&home.join("code/src/main.rs"), Some(&home), &no_cargo),
        None,
        "a Cargo.toml in the home folder is never a root"
    );
}
