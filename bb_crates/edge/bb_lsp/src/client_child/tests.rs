use std::path::PathBuf;
use std::time::{Duration, Instant};

use tokio::process::Command;

use crate::bb_client::{LspClient, Settings};

#[tokio::test]
async fn dropping_roslyn_reaps_its_descendant() {
    let file: PathBuf =
        std::env::temp_dir().join(format!("bb-lsp-descendant-{}.txt", std::process::id()));
    let mut command: Command = Command::new("sh");
    command
        .args(["-c", "sleep 30 & echo $! > \"$BB_PID_FILE\"; wait"])
        .env("BB_PID_FILE", &file);
    let client: LspClient =
        LspClient::spawn(command, Duration::from_secs(2), Settings::new()).unwrap();
    let started: Instant = Instant::now();
    while !file.exists() && started.elapsed() < Duration::from_secs(2) {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let pid: String = std::fs::read_to_string(&file).unwrap().trim().to_owned();
    drop(client);
    tokio::time::sleep(Duration::from_millis(300)).await;
    let alive: bool = std::process::Command::new("/bin/kill")
        .args(["-0", &pid])
        .status()
        .unwrap()
        .success();
    if alive {
        let _cleaned: std::io::Result<std::process::ExitStatus> =
            std::process::Command::new("/bin/kill")
                .args(["-KILL", &pid])
                .status();
    }
    let _removed: std::io::Result<()> = std::fs::remove_file(file);
    assert!(!alive, "dropped language server left descendant {pid}");
}
