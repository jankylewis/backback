//! Which Python a file uses (headmate, BBEOD28): the nearest `.venv` or
//! `venv` walking up from the file's folder (so each folder of a monorepo
//! keeps its own), else `python3` on `PATH`. A venv is a folder holding
//! `pyvenv.cfg`. backback only reads: it never creates, runs or changes a
//! venv to choose one.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The venv folder names looked for, in order, in each folder.
pub const VENV_NAMES: [&str; 2] = [".venv", "venv"];
/// The interpreter used when no venv is found.
pub const FALLBACK: &str = "python3";

/// Where the interpreter came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterpreterSource {
    /// The venv folder that holds it.
    Venv(PathBuf),
    /// `python3` found on `PATH`.
    Path,
    /// Nothing found: the bare name, left to the system to resolve.
    Bare,
}

/// A Python interpreter and why it was chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interpreter {
    /// The program to run.
    pub program: PathBuf,
    /// Where it came from.
    pub source: InterpreterSource,
}

/// The interpreter for `file` (a file or a folder): the nearest venv from
/// its folder up to `stop` (the window root, included), else `python3` on
/// `path` (the value of `PATH`).
#[must_use]
pub fn interpreter_for(file: &Path, stop: &Path, path: Option<&OsString>) -> Interpreter {
    if let Some(venv) = nearest_venv(file, stop) {
        return Interpreter {
            program: venv.join("bin").join("python"),
            source: InterpreterSource::Venv(venv),
        };
    }
    let on_path: Option<PathBuf> = path.and_then(|p: &OsString| {
        std::env::split_paths(p)
            .map(|dir: PathBuf| dir.join(FALLBACK))
            .find(|candidate: &PathBuf| candidate.is_file())
    });
    match on_path {
        Some(program) => Interpreter {
            program,
            source: InterpreterSource::Path,
        },
        None => Interpreter {
            program: PathBuf::from(FALLBACK),
            source: InterpreterSource::Bare,
        },
    }
}

/// The nearest venv folder from `file`'s folder up to `stop`.
#[must_use]
pub fn nearest_venv(file: &Path, stop: &Path) -> Option<PathBuf> {
    let start: &Path = if file.is_dir() { file } else { file.parent()? };
    for dir in start.ancestors() {
        let found: Option<PathBuf> = VENV_NAMES
            .iter()
            .map(|name: &&str| dir.join(name))
            .find(|venv: &PathBuf| venv.join("pyvenv.cfg").is_file());
        if found.is_some() {
            return found;
        }
        if dir == stop || !dir.starts_with(stop) {
            return None;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_venv_above_the_stop_folder_is_never_chosen() {
        let outer: tempfile::TempDir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(outer.path().join(".venv")).unwrap();
        std::fs::write(outer.path().join(".venv/pyvenv.cfg"), "home = x\n").unwrap();
        let root: PathBuf = outer.path().join("inner");
        std::fs::create_dir_all(&root).unwrap();
        assert_eq!(nearest_venv(&root.join("a.py"), &root), None);
    }

    #[test]
    fn a_venv_folder_without_pyvenv_cfg_is_not_a_venv() {
        let root: tempfile::TempDir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("venv/bin")).unwrap();
        assert_eq!(nearest_venv(&root.path().join("a.py"), root.path()), None);
    }

    #[test]
    fn no_venv_and_no_python_on_path_leaves_the_bare_name() {
        let empty: tempfile::TempDir = tempfile::tempdir().unwrap();
        let path: OsString = empty.path().as_os_str().to_owned();
        let chosen: Interpreter = interpreter_for(empty.path(), empty.path(), Some(&path));
        assert_eq!(chosen.program, PathBuf::from(FALLBACK));
        assert_eq!(chosen.source, InterpreterSource::Bare);
    }
}
