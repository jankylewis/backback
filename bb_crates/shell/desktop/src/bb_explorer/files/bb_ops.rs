//! The Explorer's file changes (BB-BUG-050): a new file or folder, a
//! rename, a move to the Trash. Each checks the name first and never
//! overwrites: an existing target is an error, as in VS Code.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// macOS's own Trash command (macOS 15 and later): Finder's Put Back works.
#[cfg(not(test))]
pub(crate) const TRASH: &str = "/usr/bin/trash";
/// Seam: unit tests never fill the user's Trash; `rm` stands in.
#[cfg(test)]
pub(crate) const TRASH: &str = "/bin/rm";
/// How long a move to the Trash may take before it counts as failed.
const TRASH_DEADLINE: Duration = Duration::from_secs(15);

#[derive(Debug, thiserror::Error)]
pub(crate) enum FileOpError {
    #[error("not a name: {0:?} (no '/', not empty, not . or ..)")]
    BadName(String),
    #[error("{} already exists", .0.display())]
    Exists(PathBuf),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("the Trash refused {}: {status}", .path.display())]
    Trash { path: PathBuf, status: ExitStatus },
    #[error("the Trash did not answer in {}s", TRASH_DEADLINE.as_secs())]
    TimedOut,
}

/// A checked file name: one path part, trimmed.
fn name(typed: &str) -> Result<&str, FileOpError> {
    let trimmed: &str = typed.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." || trimmed.contains(['/', '\0']) {
        return Err(FileOpError::BadName(typed.to_owned()));
    }
    Ok(trimmed)
}

/// A new empty file `typed` in `folder`.
pub(crate) fn new_file(folder: &Path, typed: &str) -> Result<PathBuf, FileOpError> {
    let path: PathBuf = folder.join(name(typed)?);
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error: std::io::Error| exists_or(error, &path))?;
    Ok(path)
}

/// A new folder `typed` in `folder`.
pub(crate) fn new_folder(folder: &Path, typed: &str) -> Result<PathBuf, FileOpError> {
    let path: PathBuf = folder.join(name(typed)?);
    std::fs::create_dir(&path).map_err(|error: std::io::Error| exists_or(error, &path))?;
    Ok(path)
}

/// `path` renamed to `typed`, in the same folder.
pub(crate) fn rename(path: &Path, typed: &str) -> Result<PathBuf, FileOpError> {
    let to: PathBuf = path.with_file_name(name(typed)?);
    if to == path {
        return Ok(to);
    }
    // A case-only rename on macOS's case-insensitive disk finds `to` there.
    let case_only: bool =
        to.to_string_lossy().to_lowercase() == path.to_string_lossy().to_lowercase();
    if !case_only && to.symlink_metadata().is_ok() {
        return Err(FileOpError::Exists(to));
    }
    std::fs::rename(path, &to)?;
    Ok(to)
}

/// Move `path` to the Trash with `program` (`TRASH`; tests pass a fake).
pub(crate) fn trash_with(program: &Path, path: &Path) -> Result<(), FileOpError> {
    let mut child: Child = Command::new(program)
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let start: Instant = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(FileOpError::Trash {
                    path: path.to_path_buf(),
                    status,
                })
            };
        }
        if start.elapsed() > TRASH_DEADLINE {
            // Our own child, one target (never a sweep).
            let _gone: std::io::Result<()> = child.kill();
            let _reaped: std::io::Result<ExitStatus> = child.wait();
            return Err(FileOpError::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn exists_or(error: std::io::Error, path: &Path) -> FileOpError {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        FileOpError::Exists(path.to_path_buf())
    } else {
        FileOpError::Io(error)
    }
}
