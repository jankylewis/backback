//! The Roslyn server logs of `bb open` and `bb probe`: one new file per run,
//! and only the newest kept (BB-PRC-006).

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::super::CommandError;

/// How many server logs the log folder keeps, newest first (BB-PRC-006).
const LOGS_KEPT: usize = 50;

/// Remove all but the newest `keep` `open-*.log` files of `log_dir`. Names
/// start with the 13-digit unix ms, so name order is age order.
fn prune_logs(log_dir: &Path, keep: usize) {
    let Ok(entries): std::io::Result<std::fs::ReadDir> = std::fs::read_dir(log_dir) else {
        return;
    };
    let mut logs: Vec<PathBuf> = entries
        .flatten()
        .map(|entry: std::fs::DirEntry| entry.path())
        .filter(|path: &PathBuf| {
            let open: bool = path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|name: &str| name.starts_with("open-"));
            let log: bool = path
                .extension()
                .is_some_and(|ext: &std::ffi::OsStr| ext.eq_ignore_ascii_case("log"));
            open && log
        })
        .collect();
    logs.sort();
    let old: usize = logs.len().saturating_sub(keep);
    for path in logs.iter().take(old) {
        // Best effort: a log that cannot go now goes on a later run.
        let _removed: std::io::Result<()> = std::fs::remove_file(path);
    }
}

/// Most names [`create_log`] tries before it gives up.
const LOG_NAME_TRIES: u32 = 1000;

/// Create this run's server log in `log_dir`: `open-<unix ms>-<pid>-<n>.log`,
/// created only if new, so two runs never write into one file.
pub(crate) fn create_log(log_dir: &Path) -> Result<(PathBuf, std::fs::File), CommandError> {
    prune_logs(log_dir, LOGS_KEPT.saturating_sub(1));
    let millis: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d: Duration| d.as_millis());
    let pid: u32 = std::process::id();
    let mut last: Option<(PathBuf, std::io::Error)> = None;
    for n in 0..LOG_NAME_TRIES {
        let path: PathBuf = log_dir.join(format!("open-{millis}-{pid}-{n}.log"));
        match std::fs::File::options()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => last = Some((path, e)),
            Err(source) => return Err(CommandError::Read { path, source }),
        }
    }
    let (path, source): (PathBuf, std::io::Error) = last.unwrap_or_else(|| {
        (
            log_dir.to_path_buf(),
            std::io::Error::from(std::io::ErrorKind::AlreadyExists),
        )
    });
    Err(CommandError::Read { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_logs_created_in_one_millisecond_never_share_a_file() {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("bb-open-logs-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (first, mut file): (PathBuf, std::fs::File) = create_log(&dir).unwrap();
        std::io::Write::write_all(&mut file, b"first run\n").unwrap();
        let (second, _other): (PathBuf, std::fs::File) = create_log(&dir).unwrap();
        let kept: String = std::fs::read_to_string(&first).unwrap();
        let _gone: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        assert_ne!(first, second);
        assert_eq!(kept, "first run\n");
    }

    #[test]
    fn the_log_folder_keeps_only_the_newest_logs() {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("bb-open-prune-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for ms in 1_700_000_000_000_u64..1_700_000_000_005 {
            std::fs::write(dir.join(format!("open-{ms}-1-0.log")), "x").unwrap();
        }
        std::fs::write(dir.join("other.txt"), "kept").unwrap();
        prune_logs(&dir, 2);
        let mut left: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e: std::io::Result<std::fs::DirEntry>| {
                e.unwrap().file_name().to_string_lossy().into_owned()
            })
            .collect();
        left.sort();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            left,
            [
                "open-1700000000003-1-0.log",
                "open-1700000000004-1-0.log",
                "other.txt"
            ]
        );
    }
}
