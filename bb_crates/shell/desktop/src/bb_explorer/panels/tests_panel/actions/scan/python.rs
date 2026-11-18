//! The Tests panel for Python (BBEOD28): the window's pytest root (its
//! `pyproject.toml`, `pytest.ini`, `setup.cfg`, `setup.py` or `tox.ini`)
//! is the project; `--collect-only -q` lists it, pytest runs it with the
//! root's interpreter, junit reports go to backback's scratch folder.

use std::path::{Path, PathBuf};

use bb_python::run::bb_pytest;
use bb_python::{PythonError, TestRoot};
use bb_testkit::{RunReport, Selection, SpecList, StopFlag};

/// The pytest root file of the window, if it has one.
pub(crate) fn root_file(root: &Path) -> Option<PathBuf> {
    bb_pytest::root_file(root)
}

/// Where `file`'s tests run: its folder, the interpreter for it, and the
/// window's scratch report folder.
fn target(root: &Path, file: &Path) -> TestRoot {
    let dir: PathBuf = file.parent().unwrap_or(root).to_path_buf();
    TestRoot {
        interpreter: bb_python::interpreter_for(&dir, root, std::env::var_os("PATH").as_ref())
            .program,
        dir,
        scratch: crate::intel::bb_worker::scratch_in("backback-python", root, "pytest"),
    }
}

/// List `file`'s tests; a missing pytest says how to install it.
pub(crate) fn list(root: &Path, file: &Path) -> Result<SpecList, String> {
    bb_pytest::list(&target(root, file), file).map_err(|e: PythonError| e.to_string())
}

/// Run `selection` of `file`'s tests.
pub(crate) fn run(
    root: &Path,
    file: &Path,
    selection: &Selection,
    stop: &StopFlag,
) -> Result<RunReport, String> {
    bb_pytest::run(&target(root, file), selection, stop).map_err(|e: PythonError| e.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::tests_panel::model::ProjectKind;

    #[test]
    fn a_pytest_root_is_a_python_row_with_no_debug() {
        for file in [
            "pyproject.toml",
            "pytest.ini",
            "setup.cfg",
            "setup.py",
            "tox.ini",
        ] {
            let kind: ProjectKind = ProjectKind::of(&Path::new("/w/shop").join(file));
            assert_eq!(
                (kind, kind.debuggable()),
                (ProjectKind::Pytest, false),
                "{file}"
            );
        }
        assert!(ProjectKind::of(Path::new("/w/App.Tests.csproj")).debuggable());
    }
}
