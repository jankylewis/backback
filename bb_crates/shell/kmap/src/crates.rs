//! The workspace: its member crates, what each says it is, which internal
//! crates it uses, and its size. Read from the Cargo.toml files, std only.

use std::path::{Path, PathBuf};

use crate::{read, KmapError};

/// One member crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crate {
    /// Package name.
    pub name: String,
    /// Folder, relative to the workspace root.
    pub path: PathBuf,
    /// The package `description`.
    pub description: Option<String>,
    /// Internal (`bb-*`) crates under `[dependencies]`.
    pub uses: Vec<String>,
    /// `.rs` files.
    pub files: usize,
    /// Lines in those files.
    pub lines: usize,
}

/// The members of the workspace at `root`, in `members` order.
///
/// # Errors
/// [`KmapError::NoWorkspace`] without a members list, [`KmapError::Read`]
/// when a manifest or source file cannot be read.
pub fn workspace(root: &Path) -> Result<Vec<Crate>, KmapError> {
    let manifest: String = read(&root.join("Cargo.toml"))
        .map_err(|_: KmapError| KmapError::NoWorkspace(root.to_path_buf()))?;
    let members: Vec<String> = members(&manifest);
    if members.is_empty() {
        return Err(KmapError::NoWorkspace(root.to_path_buf()));
    }
    members
        .iter()
        .map(|member: &String| member_crate(root, member))
        .collect()
}

fn members(manifest: &str) -> Vec<String> {
    let Some((_, after)): Option<(&str, &str)> = manifest.split_once("members = [") else {
        return Vec::new();
    };
    let list: &str = after.split(']').next().unwrap_or("");
    list.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

fn member_crate(root: &Path, member: &str) -> Result<Crate, KmapError> {
    let dir: PathBuf = root.join(member);
    let manifest: String = read(&dir.join("Cargo.toml"))?;
    let mut lines: usize = 0;
    let files: Vec<PathBuf> = rust_files(&dir)?;
    for file in &files {
        lines = lines.saturating_add(read(file)?.lines().count());
    }
    Ok(Crate {
        name: value(&manifest, "name").unwrap_or_else(|| member.to_owned()),
        path: PathBuf::from(member),
        description: value(&manifest, "description"),
        uses: uses(&manifest),
        files: files.len(),
        lines,
    })
}

/// `key = "value"` on its own line.
fn value(manifest: &str, key: &str) -> Option<String> {
    manifest.lines().find_map(|line: &str| {
        let rest: &str = line
            .trim()
            .strip_prefix(key)?
            .trim_start()
            .strip_prefix('=')?;
        let quoted: &str = rest.trim().strip_prefix('"')?;
        quoted
            .rsplit_once('"')
            .map(|(v, _): (&str, &str)| v.to_owned())
    })
}

fn uses(manifest: &str) -> Vec<String> {
    let mut section: &str = "";
    let mut out: Vec<String> = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            section = line;
        } else if section == "[dependencies]" && line.starts_with("bb-") {
            out.push(
                line.split(['.', ' ', '='])
                    .next()
                    .unwrap_or(line)
                    .to_owned(),
            );
        }
    }
    out
}

/// Every `.rs` file under `dir` (a file is its own list), sorted, skipping
/// `target/` and dot folders.
///
/// # Errors
/// [`KmapError::Read`] when a folder cannot be listed.
pub fn rust_files(dir: &Path) -> Result<Vec<PathBuf>, KmapError> {
    if dir.is_file() {
        return Ok(vec![dir.to_path_buf()]);
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|source: std::io::Error| KmapError::Read {
            path: dir.to_path_buf(),
            source,
        })?
        .filter_map(|e: std::io::Result<std::fs::DirEntry>| {
            e.ok().map(|e: std::fs::DirEntry| e.path())
        })
        .collect();
    entries.sort();
    for path in entries {
        let name: &str = path
            .file_name()
            .and_then(|n: &std::ffi::OsStr| n.to_str())
            .unwrap_or("");
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if path.is_dir() {
            out.extend(rust_files(&path)?);
        } else if path
            .extension()
            .is_some_and(|x: &std::ffi::OsStr| x == "rs")
        {
            out.push(path);
        }
    }
    Ok(out)
}
