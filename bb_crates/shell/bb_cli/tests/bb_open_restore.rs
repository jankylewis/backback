//! `bb open` never writes into the opened solution unless `--restore` is given.
//! Runs the real Roslyn server on a fresh copy of the harness.

#[cfg(test)]
mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

use common::{bb, harness, Run};
use open_copy::{fresh_copy, log_of, open, snapshot};

#[cfg(test)]
mod open_copy {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;

    use crate::common::Run;

    /// A fresh, never-restored copy of the harness under the temp folder.
    pub fn fresh_copy(name: &str) -> PathBuf {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("bb-e2e-open-{name}-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        copy_tree(&crate::common::harness(), &dir);
        dir
    }

    fn copy_tree(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let path: PathBuf = entry.unwrap().path();
            let target: PathBuf = to.join(path.file_name().unwrap());
            let name: String = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() && name != "obj" && name != "bin" {
                copy_tree(&path, &target);
            } else if path.is_file() {
                std::fs::copy(&path, &target).unwrap();
            }
        }
    }

    /// Every file and folder below `root` with its size and modified time
    /// (a new entry also changes its folder's time).
    pub fn snapshot(root: &Path) -> BTreeMap<PathBuf, (u64, SystemTime)> {
        let mut files: BTreeMap<PathBuf, (u64, SystemTime)> = BTreeMap::new();
        for entry in std::fs::read_dir(root).unwrap() {
            let path: PathBuf = entry.unwrap().path();
            let meta: std::fs::Metadata = std::fs::metadata(&path).unwrap();
            if path.is_dir() {
                files.extend(snapshot(&path));
                files.insert(path, (0, meta.modified().unwrap()));
            } else {
                files.insert(path, (meta.len(), meta.modified().unwrap()));
            }
        }
        files
    }

    /// `bb open` on the copy in `dir`; it must succeed.
    pub fn open(dir: &Path, extra: &[&str]) -> Run {
        let sln: PathBuf = dir.join("Harness.slnx");
        let mut args: Vec<&str> = vec!["open", sln.to_str().unwrap()];
        args.extend_from_slice(extra);
        let run: Run = crate::common::bb(&args);
        assert_eq!(run.code, Some(0), "{}", run.all());
        run
    }

    /// The server log `bb open` printed.
    pub fn log_of(run: &Run) -> String {
        let path: &str = run
            .out
            .lines()
            .find_map(|l: &str| l.strip_prefix("log       : "))
            .unwrap_or_else(|| panic!("no log line:\n{}", run.all()));
        std::fs::read_to_string(path).unwrap()
    }
}

#[test]
fn open_by_default_tells_roslyn_not_to_restore_and_writes_nothing_in_the_solution() {
    let dir: PathBuf = fresh_copy("norestore");
    let before: BTreeMap<PathBuf, (u64, SystemTime)> = snapshot(&dir);
    let run: Run = open(&dir, &[]);
    let log: String = log_of(&run);
    let after: BTreeMap<PathBuf, (u64, SystemTime)> = snapshot(&dir);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    assert!(
        log.contains("[config] projects.dotnet_enable_automatic_restore"),
        "{log}"
    );
    assert!(!log.contains("Running dotnet restore"), "{log}");
    assert_eq!(after, before);
}

#[test]
fn open_with_restore_flag_lets_roslyn_restore() {
    let dir: PathBuf = fresh_copy("restore");
    let run: Run = open(&dir, &["--restore"]);
    let log: String = log_of(&run);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    assert!(log.contains("Running dotnet restore"), "{log}");
}

#[test]
fn open_rejects_an_unknown_flag() {
    let sln: PathBuf = harness().join("Harness.slnx");
    let run: Run = bb(&["open", sln.to_str().unwrap(), "--fast"]);
    assert_eq!(run.code, Some(2), "{}", run.all());
}
