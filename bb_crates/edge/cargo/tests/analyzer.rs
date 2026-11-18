//! rust-analyzer discovery: override, then PATH, then rustup; the rustup
//! shim that exists without the component is not a server.
#![cfg(test)]

use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use bb_cargo::analyzer::{self, Found, Inputs, Source, MISSING_HINT};
use bb_cargo::CargoError;

const SHIM: &str =
    "#!/bin/sh\necho \"error: Unknown binary 'rust-analyzer' in official toolchain\" >&2\nexit 1\n";
const SERVER: &str = "#!/bin/sh\necho \"rust-analyzer 1.99.1 (fake)\"\n";

fn script(dir: &Path, text: &str) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    let path: PathBuf = dir.join(analyzer::BINARY);
    std::fs::write(&path, text).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn inputs(env: Option<&Path>, path: &[&Path]) -> Inputs {
    Inputs {
        env: env.map(|p: &Path| p.as_os_str().to_owned()),
        path: Some(std::env::join_paths(path).unwrap()),
        rustup: false,
    }
}

#[test]
fn the_rustup_shim_alone_on_path_is_missing_with_the_install_hint() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let shim_dir: PathBuf = base.path().join("shim");
    script(&shim_dir, SHIM);
    let result: Result<Found, CargoError> = analyzer::locate(&inputs(None, &[&shim_dir]));
    assert!(
        matches!(result, Err(CargoError::AnalyzerMissing)),
        "{result:?}"
    );
    assert_eq!(
        CargoError::AnalyzerMissing.to_string(),
        "rust-analyzer not installed: rustup component add rust-analyzer"
    );
    assert_eq!(MISSING_HINT, CargoError::AnalyzerMissing.to_string());
}

#[test]
fn a_server_on_path_after_the_shim_is_found_by_its_version() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let shim_dir: PathBuf = base.path().join("shim");
    let real_dir: PathBuf = base.path().join("real");
    script(&shim_dir, SHIM);
    let real: PathBuf = script(&real_dir, SERVER);
    let found: Found = analyzer::locate(&inputs(None, &[&real_dir, &shim_dir])).unwrap();
    assert_eq!(
        (found.binary, found.source, found.version.as_str()),
        (real, Source::Path, "rust-analyzer 1.99.1 (fake)")
    );
}

#[test]
fn the_override_wins_over_path() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let chosen: PathBuf = script(&base.path().join("chosen"), SERVER);
    let other_dir: PathBuf = base.path().join("other");
    script(&other_dir, SERVER);
    let found: Found = analyzer::locate(&inputs(Some(&chosen), &[&other_dir])).unwrap();
    assert_eq!((found.binary, found.source), (chosen, Source::Env));
}

#[test]
fn an_override_that_does_not_run_is_reported_never_skipped() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let broken: PathBuf = script(&base.path().join("broken"), SHIM);
    let real_dir: PathBuf = base.path().join("real");
    script(&real_dir, SERVER);
    let result: Result<Found, CargoError> = analyzer::locate(&inputs(Some(&broken), &[&real_dir]));
    assert!(
        matches!(&result, Err(CargoError::AnalyzerBroken { binary, .. }) if *binary == broken),
        "{result:?}"
    );
}

#[test]
fn the_server_builds_into_the_scratch_folder_never_the_project_target() {
    let scratch: &Path = Path::new("/scratch/backback-rust/target");
    let command: std::process::Command = analyzer::command(Path::new("/bin/ra"), scratch);
    let env: Vec<(OsString, Option<OsString>)> = command
        .get_envs()
        .map(|(k, v): (&std::ffi::OsStr, Option<&std::ffi::OsStr>)| {
            (k.to_owned(), v.map(std::ffi::OsStr::to_owned))
        })
        .collect();
    assert_eq!(
        env,
        vec![(
            OsString::from("CARGO_TARGET_DIR"),
            Some(scratch.as_os_str().to_owned())
        )]
    );
    let settings: serde_json::Value = analyzer::settings(scratch);
    assert_eq!(settings["cargo"]["targetDir"], scratch.to_str().unwrap());
}

#[test]
fn this_machine_either_runs_rust_analyzer_or_names_the_install_hint() {
    let found: Result<Found, CargoError> = analyzer::locate(&Inputs::from_env());
    match found {
        Ok(server) => assert!(server.version.starts_with("rust-analyzer"), "{server:?}"),
        Err(error) => {
            let said: String = error.to_string();
            assert!(
                said == MISSING_HINT || said.contains("does not run"),
                "{said}"
            );
        }
    }
}
