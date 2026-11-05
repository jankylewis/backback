//! The Explorer's file changes (BB-BUG-050, BB-BUG-052): a new file or
//! folder, a rename, a move to the Trash, a paste of a copy. Each checks the
//! name first and never overwrites: an existing target is an error, and a
//! paste takes a free `name copy` name, as in VS Code.

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
    #[error("{} cannot be pasted into itself", .0.display())]
    IntoItself(PathBuf),
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

/// A copy of `from` (file or whole folder) in `folder`, under its own name
/// or the first free `name copy`, `name copy 2`, ...; links stay links.
pub(crate) fn copy_into(from: &Path, folder: &Path) -> Result<PathBuf, FileOpError> {
    let meta: std::fs::Metadata = from.symlink_metadata()?;
    if meta.is_dir() && folder.starts_with(from) {
        return Err(FileOpError::IntoItself(from.to_path_buf()));
    }
    let to: PathBuf = free_name(from, folder);
    copy_tree(from, &to)?;
    Ok(to)
}

/// `folder/name`, else the first free `name copy.ext`, `name copy 2.ext`, ...
/// (10,000 tries; then `name` itself, which the copy refuses as existing).
fn free_name(from: &Path, folder: &Path) -> PathBuf {
    let own: PathBuf = folder.join(from.file_name().unwrap_or_default());
    if own.symlink_metadata().is_err() {
        return own;
    }
    let stem: String = from
        .file_stem()
        .map(|s: &std::ffi::OsStr| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext: String = from
        .extension()
        .map(|e: &std::ffi::OsStr| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (1_u32..=10_000)
        .map(|n: u32| match n {
            1 => folder.join(format!("{stem} copy{ext}")),
            _ => folder.join(format!("{stem} copy {n}{ext}")),
        })
        .find(|p: &PathBuf| p.symlink_metadata().is_err())
        .unwrap_or(own)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), FileOpError> {
    let meta: std::fs::Metadata = from.symlink_metadata()?;
    if meta.file_type().is_symlink() {
        std::os::unix::fs::symlink(std::fs::read_link(from)?, to)?;
    } else if meta.is_dir() {
        std::fs::create_dir(to).map_err(|error: std::io::Error| exists_or(error, to))?;
        for entry in std::fs::read_dir(from)? {
            let entry: std::fs::DirEntry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else {
        if to.symlink_metadata().is_ok() {
            return Err(FileOpError::Exists(to.to_path_buf()));
        }
        let _bytes: u64 = std::fs::copy(from, to)?;
    }
    Ok(())
}

fn exists_or(error: std::io::Error, path: &Path) -> FileOpError {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        FileOpError::Exists(path.to_path_buf())
    } else {
        FileOpError::Io(error)
    }
}
