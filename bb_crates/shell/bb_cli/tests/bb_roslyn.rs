//! The real Roslyn server behind `bb open` and `bb probe` (restore off).
//! `bb open` never writes into the opened solution unless `--restore` is given.

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
fn open_waits_until_every_project_of_the_solution_has_loaded() {
    let dir: PathBuf = fresh_copy("allprojects");
    let run: Run = open(&dir, &[]);
    let log: String = log_of(&run);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    let loaded: usize = log
        .lines()
        .filter_map(|l: &str| l.split("Completed (re)load of ").nth(1))
        .filter_map(|rest: &str| rest.split(' ').next()?.parse::<usize>().ok())
        .sum();
    assert_eq!(loaded, 4, "{log}");
    assert!(
        run.out.contains("until all 4 projects loaded"),
        "{}",
        run.out
    );
}

#[test]
fn open_keeps_the_server_lines_until_the_server_has_shut_down() {
    let dir: PathBuf = fresh_copy("shutdownlog");
    let run: Run = open(&dir, &[]);
    let log: String = log_of(&run);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    assert!(log.contains("[info] Shutting down"), "{log}");
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

fn spec() -> String {
    harness()
        .join("_NUnitApex/GreeterSpec.cs")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn probe_definition_finds_the_class_in_core_and_times_warm_requests() {
    let run: Run = bb(&["probe", &spec(), "13:10", "definition", "--repeat", "3"]);
    assert_eq!(run.code, Some(0), "{}", run.all());
    assert!(run.out.contains("result    : 1 locations"), "{}", run.out);
    assert!(run.out.contains("  Core/Greeter.cs:3:21\n"), "{}", run.out);
    assert!(run.out.contains("first     : "), "{}", run.out);
    assert!(run.out.contains("warm      : n=2 p50 "), "{}", run.out);
}

#[test]
fn probe_leaves_the_solution_tree_unchanged() {
    let dir: PathBuf = fresh_copy("probe-trace");
    let before: BTreeMap<PathBuf, (u64, SystemTime)> = snapshot(&dir);
    let file: String = dir
        .join("_NUnitApex/GreeterSpec.cs")
        .to_string_lossy()
        .into_owned();
    let run: Run = bb(&["probe", &file, "13:10", "definition"]);
    let after: BTreeMap<PathBuf, (u64, SystemTime)> = snapshot(&dir);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    assert_eq!(run.code, Some(0), "{}", run.all());
    assert_eq!(after, before);
}

#[test]
fn probe_rename_lists_three_files_and_changes_none() {
    let core: PathBuf = harness().join("Core/Greeter.cs");
    let before: String = std::fs::read_to_string(&core).unwrap();
    let run: Run = bb(&["probe", &spec(), "13:10", "rename=Welcomer"]);
    let after: String = std::fs::read_to_string(&core).unwrap();
    assert_eq!(run.code, Some(0), "{}", run.all());
    assert!(run.out.contains("result    : 3 files"), "{}", run.out);
    assert!(
        run.out.contains("  McpServer/GreetTool.cs ("),
        "{}",
        run.out
    );
    assert_eq!(after, before);
}

#[test]
fn probe_refuses_a_position_outside_the_file_and_a_bad_feature() {
    let outside: Run = bb(&["probe", &spec(), "99:1", "hover"]);
    let feature: Run = bb(&["probe", &spec(), "1:1", "fly"]);
    assert_eq!(outside.code, Some(1), "{}", outside.all());
    assert!(
        outside.err.contains("line 99 column 1 is not in the file"),
        "{}",
        outside.err
    );
    assert_eq!(feature.code, Some(2), "{}", feature.all());
}
