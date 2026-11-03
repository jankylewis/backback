//! Own each temp folder: none may outlive its process (BB-PRC-004).
//!
//! A folder is named `<prefix>-<pid>` or `<prefix>-<pid>-<suffix>` under the
//! OS temp folder; test fixtures put a name between (`<prefix>-<name>-<pid>`),
//! so a sweep takes the first all-digit part after the prefix as the pid. Drop and app quit ([`crate::shutdown`]) remove it; a
//! crash or a static that never drops leaves it, so [`OwnedTempDir::create`]
//! first sweeps the folders of its prefix whose process is gone.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// Every prefix backback and its tests name temp folders with.
pub const FAMILIES: [&str; 9] = [
    "backback-intel",
    "backback-testhost",
    "bb-e2e",
    "bb-list",
    "bb-run",
    "bb-testkit",
    "bb-intel",
    "bb-gui",
    "bb-process",
];

static OWNED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
/// Deadline for one `/bin/ps -p` call.
const PROBE_DEADLINE: Duration = Duration::from_millis(500);

/// A temp folder this process owns.
#[derive(Debug)]
pub struct OwnedTempDir(PathBuf);

impl OwnedTempDir {
    /// Sweep `prefix`'s dead folders, then make `<prefix>-<pid>` afresh.
    ///
    /// # Errors
    /// The folder cannot be made.
    pub fn create(prefix: &str) -> std::io::Result<Self> {
        Self::make(prefix, format!("{prefix}-{}", std::process::id()))
    }

    /// [`Self::create`] for `<prefix>-<pid>-<suffix>`: several per process.
    ///
    /// # Errors
    /// The folder cannot be made.
    pub fn create_with(prefix: &str, suffix: &str) -> std::io::Result<Self> {
        Self::make(prefix, format!("{prefix}-{}-{suffix}", std::process::id()))
    }

    fn make(prefix: &str, name: String) -> std::io::Result<Self> {
        let _swept: usize = sweep(prefix);
        let path: PathBuf = std::env::temp_dir().join(name);
        // A folder of this very name is this process's own earlier one.
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)?;
        owned().push(path.clone());
        Ok(Self(path))
    }

    /// The folder.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for OwnedTempDir {
    fn drop(&mut self) {
        owned().retain(|path: &PathBuf| path != &self.0);
        // Best effort: what cannot be removed now is swept on the next start.
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(&self.0);
    }
}

fn owned() -> MutexGuard<'static, Vec<PathBuf>> {
    OWNED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Remove every folder this process still owns (app quit).
pub fn remove_owned() {
    let paths: Vec<PathBuf> = std::mem::take(&mut *owned());
    for path in paths {
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(&path);
    }
}

/// [`sweep`] every family; how many folders were removed.
#[must_use]
pub fn sweep_all() -> usize {
    FAMILIES
        .iter()
        .map(|prefix: &&str| sweep(prefix))
        .fold(0, usize::saturating_add)
}

/// Remove `prefix`'s folders whose process is gone; how many were removed.
/// A folder whose pid is alive, or that cannot be checked, is kept.
#[must_use]
pub fn sweep(prefix: &str) -> usize {
    let Ok(entries): std::io::Result<std::fs::ReadDir> = std::fs::read_dir(std::env::temp_dir())
    else {
        return 0;
    };
    let own: u32 = std::process::id();
    let mut removed: usize = 0;
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(pid): Option<u32> = owner(&name, prefix) else {
            continue;
        };
        if pid != own && !alive(pid) && std::fs::remove_dir_all(entry.path()).is_ok() {
            removed = removed.saturating_add(1);
        }
    }
    removed
}

/// The pid of `<prefix>-[<name>-...]<pid>[-<suffix>]`: the first part
/// after the prefix made only of digits.
fn owner(name: &str, prefix: &str) -> Option<u32> {
    let rest: &str = name.strip_prefix(prefix)?.strip_prefix('-')?;
    rest.split('-')
        .find(|part: &&str| !part.is_empty() && part.bytes().all(|b: u8| b.is_ascii_digit()))?
        .parse()
        .ok()
}

/// Whether process `pid` exists, whoever owns it (`ps -p`; `kill -0`
/// fails on another user's process). A probe that cannot run says alive,
/// so nothing is removed on doubt.
fn alive(pid: u32) -> bool {
    let spawned: std::io::Result<std::process::Child> = Command::new("/bin/ps")
        .args(["-p", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut probe): std::io::Result<std::process::Child> = spawned else {
        return true;
    };
    let deadline: Instant = Instant::now() + PROBE_DEADLINE;
    loop {
        match probe.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
            Ok(None) | Err(_) => {
                let _stopped: std::io::Result<()> = probe.kill();
                let _reaped: std::io::Result<std::process::ExitStatus> = probe.wait();
                return true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_prefix_then_a_pid_names_an_owner() {
        assert_eq!(owner("bb-x-123", "bb-x"), Some(123));
        assert_eq!(owner("bb-x-123-ThreadId(2)", "bb-x"), Some(123));
        assert_eq!(owner("bb-x-e2e-123", "bb-x"), Some(123), "a fixture name");
        assert_eq!(owner("bb-x-e2e", "bb-x"), None, "no pid");
        assert_eq!(owner("bb-xy-123", "bb-x"), None);
        assert_eq!(owner("bb-x-", "bb-x"), None);
        assert_eq!(owner("bb-x-12a", "bb-x"), None);
    }

    #[test]
    fn a_folder_goes_on_drop_and_a_dead_owners_folder_on_the_next_create() {
        let prefix: &str = "bb-process-selftest";
        // A pid no process has: the probe says gone.
        let dead: PathBuf = std::env::temp_dir().join(format!("{prefix}-99999999"));
        std::fs::create_dir_all(dead.join("deep")).unwrap();
        let live: PathBuf = std::env::temp_dir().join(format!("{prefix}-1"));
        std::fs::create_dir_all(&live).unwrap();
        let dir: OwnedTempDir = OwnedTempDir::create(prefix).unwrap();
        assert!(dir.path().is_dir());
        assert!(!dead.exists(), "a dead owner's folder is swept");
        assert!(live.exists(), "launchd (pid 1) is alive: kept");
        let path: PathBuf = dir.path().to_path_buf();
        drop(dir);
        assert!(!path.exists(), "drop removes the folder");
        std::fs::remove_dir_all(live).unwrap();
    }

    #[test]
    fn quit_removes_what_is_still_owned() {
        let dir: OwnedTempDir = OwnedTempDir::create_with("bb-process-quit", "a").unwrap();
        let path: PathBuf = dir.path().to_path_buf();
        remove_owned();
        assert!(!path.exists());
        drop(dir);
    }
}
