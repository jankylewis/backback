use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;

pub struct Built {
    pub dir: PathBuf,
    /// Holds the folder; a static `Built` never drops, so the next run's
    /// create sweeps it once this process is gone (BB-PRC-004).
    _owned: bb_process::OwnedTempDir,
}

/// Build the fixture in a temp folder of this process's own,
/// `bb-dap-<pid>-<tag>`: test processes run side by side (CI's parallel
/// workers) must never share one, else one deletes or locks the files
/// another is building.
pub async fn build(tag: &str) -> Built {
    let fixture: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture");
    let owned: bb_process::OwnedTempDir =
        bb_process::OwnedTempDir::create_with("bb-dap", tag).unwrap();
    let scratch: PathBuf = owned.path().to_path_buf();
    for entry in std::fs::read_dir(&fixture).unwrap() {
        let path: PathBuf = entry.unwrap().path();
        if path.is_file() {
            std::fs::copy(&path, scratch.join(path.file_name().unwrap())).unwrap();
        }
    }
    let output: std::process::Output = tokio::time::timeout(
        Duration::from_secs(120),
        Command::new("dotnet")
            .arg("build")
            .arg(scratch.join("GotoFixture.csproj"))
            .arg("-c")
            .arg("Debug")
            .kill_on_drop(true)
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        output.status.success(),
        "fixture build failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let dir: PathBuf = scratch.canonicalize().unwrap();
    Built { dir, _owned: owned }
}
