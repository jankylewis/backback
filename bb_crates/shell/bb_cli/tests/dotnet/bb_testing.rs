//! `bb test list` and `bb test run` on a built copy of `fixtures/_Harness`,
//! through the pinned Apex CLI and plain `dotnet`.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use crate::common::{bb, harness, Run};

static BUILT: OnceLock<PathBuf> = OnceLock::new();

const NUNIT: &str = "_NUnitApex/_NUnitApex.csproj";

/// A copy of the harness with both test projects built.
fn built() -> &'static Path {
    BUILT.get_or_init(|| {
        let dir: PathBuf = std::env::temp_dir().join(format!("bb-e2e-test-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        copy_tree(&harness(), &dir);
        for project in [NUNIT, "XunitPlaywright/_XunitPlaywright.csproj"] {
            let status: std::process::ExitStatus = std::process::Command::new("dotnet")
                .args(["build", project, "-c", "Debug", "-nologo", "-v", "q"])
                .current_dir(&dir)
                .status()
                .unwrap();
            assert!(status.success(), "build of {project} failed");
        }
        std::fs::canonicalize(dir).unwrap()
    })
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

/// `BB_APEX_COMMAND`, else the pinned Apex CLI from the NuGet cache.
fn apex_command() -> String {
    std::env::var("BB_APEX_COMMAND").unwrap_or_else(|_: std::env::VarError| {
        let dll: PathBuf = PathBuf::from(std::env::var("HOME").unwrap())
            .join(".nuget/packages/apex.cli/0.8.2-pre/tools/net10.0/any/apex.dll");
        assert!(
            dll.is_file(),
            "apex.cli 0.8.2-pre is not installed: {}",
            dll.display()
        );
        format!("dotnet {}", dll.display())
    })
}

fn path_of(relative: &str) -> String {
    built().join(relative).to_string_lossy().into_owned()
}

#[test]
fn list_leaves_the_built_solution_tree_unchanged() {
    let root: PathBuf = std::env::temp_dir().join(format!(
        "bb-list-trace-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    copy_tree(&harness(), &root);
    for project in [NUNIT, "XunitPlaywright/_XunitPlaywright.csproj"] {
        let built: std::process::ExitStatus = std::process::Command::new("dotnet")
            .args(["build", project, "-c", "Debug", "-nologo", "-v", "q"])
            .current_dir(&root)
            .status()
            .unwrap();
        assert!(built.success(), "build of {project}");
    }
    let marker: PathBuf = root.join("bb-list-marker");
    std::fs::write(&marker, "before list").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let before: SystemTime = SystemTime::now();
    let nunit: String = root.join(NUNIT).to_string_lossy().into_owned();
    let xunit: String = root
        .join("XunitPlaywright/_XunitPlaywright.csproj")
        .to_string_lossy()
        .into_owned();
    let apex: String = apex_command();
    for args in [
        vec!["test", "list", &nunit, "--backend", "dotnet"],
        vec!["test", "list", &xunit, "--backend", "dotnet"],
        vec!["test", "list", &nunit, "--apex-command", &apex],
    ] {
        let listed: Run = bb(&args);
        assert_eq!(listed.code, Some(0), "{}", listed.all());
    }
    let changed: Vec<PathBuf> = newer_files(&root, before);
    assert!(changed.is_empty(), "list wrote in solution: {changed:?}");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn run_puts_runner_output_only_in_temp() {
    let root: PathBuf = std::env::temp_dir().join(format!(
        "bb-run-trace-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    copy_tree(&harness(), &root);
    let built: std::process::ExitStatus = std::process::Command::new("dotnet")
        .args(["build", NUNIT, "-c", "Debug", "-nologo", "-v", "q"])
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(built.success());
    let marker: PathBuf = root.join("bb-run-marker");
    std::fs::write(&marker, "before run").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let before: SystemTime = SystemTime::now();
    let project: String = root.join(NUNIT).to_string_lossy().into_owned();
    let run: Run = bb(&[
        "test",
        "run",
        &project,
        "--backend",
        "dotnet",
        "--category",
        "Smoke",
    ]);
    assert_eq!(run.code, Some(0), "{}", run.all());
    let changed: Vec<PathBuf> = newer_files(&root, before);
    assert!(
        changed.is_empty(),
        "run output entered solution: {changed:?}"
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn newer_files(root: &Path, before: SystemTime) -> Vec<PathBuf> {
    let mut changed: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path: PathBuf = entry.unwrap().path();
        if path.is_dir() {
            changed.extend(newer_files(&path, before));
        } else if path.metadata().unwrap().modified().unwrap() > before {
            changed.push(path);
        }
    }
    changed
}

#[test]
fn list_prints_each_test_project_as_a_tree_with_its_backend_and_time() {
    let command: String = apex_command();
    let run: Run = bb(&[
        "test",
        "list",
        &path_of("Harness.slnx"),
        "--apex-command",
        &command,
    ]);
    assert_eq!(run.code, Some(0), "{}", run.all());
    for want in [
        "_NUnitApex  [apex] 6 specs, list ",
        "_XunitPlaywright  [mtp] 1 specs, list ",
        "  Harness.NUnitApex\n    GreeterSpec\n      Greets(\"Vinh\",\"hi Vinh\")\n",
        "    OutcomeSpec\n      Fails\n      Passes\n",
        "listed    : _NUnitApex [apex] 6 specs in ",
        "total     : 2 projects in ",
    ] {
        assert!(run.out.contains(want), "no {want:?} in:\n{}", run.out);
    }
}

#[test]
fn run_prints_each_outcome_errors_a_missing_spec_and_never_a_secret() {
    let command: String = apex_command();
    let specs: &str = "Harness.NUnitApex.OutcomeSpec.Passes,Harness.NUnitApex.OutcomeSpec.Fails,Harness.NUnitApex.OutcomeSpec.Skipped,Harness.NUnitApex.OutcomeSpec.Gone";
    let run: Run = bb(&[
        "test",
        "run",
        &path_of(NUNIT),
        "--apex-command",
        &command,
        "--specs",
        specs,
    ]);
    assert_eq!(run.code, Some(1), "{}", run.all());
    for want in [
        "settings  : rs harness.runsettings: 3 vars (1 secret, workers 2, UNSET HARNESS_TOTP_KEY)",
        "_NUnitApex  [apex] run ",
        "      PASS  Passes  ",
        "      FAIL  Fails  ",
        "  two is not three",
        "      SKIP  Skipped  ",
        "      ERROR Gone  not in TRX",
        "summary   : 1 passed, 1 failed, 1 skipped, 1 errored",
    ] {
        assert!(run.out.contains(want), "no {want:?} in:\n{}", run.out);
    }
    assert!(run.err.contains("bb: 1 failed, 1 errored"), "{}", run.err);
    assert!(
        !run.all().contains("fixture-not-a-real-secret"),
        "secret printed"
    );
}

#[test]
fn run_with_dotnet_and_a_category_runs_only_that_category() {
    let run: Run = bb(&[
        "test",
        "run",
        &path_of(NUNIT),
        "--backend",
        "dotnet",
        "--category",
        "Smoke",
    ]);
    assert_eq!(run.code, Some(0), "{}", run.all());
    assert!(run.out.contains("_NUnitApex  [vstest] run "), "{}", run.out);
    assert!(run.out.contains("      PASS  Passes  "), "{}", run.out);
    assert!(
        run.out
            .contains("summary   : 1 passed, 0 failed, 0 skipped, 0 errored"),
        "{}",
        run.out
    );
}
