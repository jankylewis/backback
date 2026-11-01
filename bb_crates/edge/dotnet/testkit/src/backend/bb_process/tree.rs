//! Stop a process and everything it started: killing only the program
//! leaves its children running (`dotnet test` keeps a test host alive).

use std::time::Duration;

use tokio::process::{Child, Command};
use tokio::time::error::Elapsed;

/// Deadline for `pgrep` and `kill` while stopping a tree.
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

/// Kill `child` and every process below it, all at once. Children are
/// found while the parent still lives (an orphan is no longer findable),
/// and killed together so no parent can start a replacement (`dotnet test`
/// starts a new test host when its host dies).
pub(crate) async fn stop_tree(child: &mut Child) {
    let Some(root): Option<u32> = child.id() else {
        return;
    };
    let mut all: Vec<u32> = vec![root];
    let mut level: Vec<u32> = vec![root];
    while !level.is_empty() {
        level = children_of(&level).await;
        all.extend(&level);
    }
    let ids: Vec<String> = all.iter().map(u32::to_string).collect();
    let mut kill: Command = Command::new("kill");
    kill.arg("-KILL").args(&ids).kill_on_drop(true);
    // Best effort: a process that already ended cannot be killed again.
    let _killed: Result<std::io::Result<std::process::Output>, Elapsed> =
        tokio::time::timeout(STOP_TIMEOUT, kill.output()).await;
    let _reaped: std::io::Result<()> = child.kill().await;
}

async fn children_of(parents: &[u32]) -> Vec<u32> {
    let list: String = parents
        .iter()
        .map(u32::to_string)
        .collect::<Vec<String>>()
        .join(",");
    let mut pgrep: Command = Command::new("pgrep");
    pgrep.args(["-P", &list]).kill_on_drop(true);
    match tokio::time::timeout(STOP_TIMEOUT, pgrep.output()).await {
        Ok(Ok(out)) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l: &str| l.trim().parse::<u32>().ok())
            .collect(),
        Ok(Err(_)) | Err(_) => Vec::new(),
    }
}
