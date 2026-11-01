use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;

pub struct Built {
    pub dir: PathBuf,
}

pub async fn build(tag: &str) -> Built {
    let fixture: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture");
    let scratch: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("backback-{tag}"));
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).unwrap();
    }
    std::fs::create_dir_all(&scratch).unwrap();
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
    Built { dir }
}
