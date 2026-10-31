//! Folder rules (HD taxonomy, underscore) and file line caps.

use std::path::{Path, PathBuf};

use crate::{caps, csharp, rust, AuditError, Finding, Rule, Violation};

/// Folders that are never audited: VCS, build output, dependencies.
const SKIP_DIRS: [&str; 6] = [".git", ".target", "target", "bin", "obj", "node_modules"];
/// File endings that mark a C# tree.
const CSHARP_MARKERS: [&str; 4] = [".cs", ".csproj", ".slnx", ".sln"];
/// Generated or legal files that are exempt from line caps.
const SKIP_FILES: [&str; 2] = ["Cargo.lock", "LICENSE"];

pub(crate) fn walk(root: &Path) -> Result<Vec<Violation>, AuditError> {
    let mut out: Vec<Violation> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries: Vec<PathBuf> = read_dir(&dir)?;
        check_folder(root, &dir, &entries, &mut out);
        for entry in entries {
            let name: String = file_name(&entry);
            if entry.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(entry);
                }
            } else if !SKIP_FILES.contains(&name.as_str()) {
                check_file(root, &entry, &mut out)?;
            }
        }
    }
    Ok(out)
}

fn read_dir(dir: &Path) -> Result<Vec<PathBuf>, AuditError> {
    let mut entries: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e: std::io::Error| read_error(dir, e))? {
        entries.push(
            entry
                .map_err(|e: std::io::Error| read_error(dir, e))?
                .path(),
        );
    }
    entries.sort();
    Ok(entries)
}

fn read_error(path: &Path, source: std::io::Error) -> AuditError {
    AuditError::Read {
        path: path.to_path_buf(),
        source,
    }
}

fn check_folder(root: &Path, dir: &Path, entries: &[PathBuf], out: &mut Vec<Violation>) {
    let visible: Vec<String> = entries
        .iter()
        .map(|p: &PathBuf| file_name(p))
        .filter(|n: &String| !n.starts_with('.'))
        .collect();
    if visible.len() > caps::FOLDER_ENTRIES {
        let detail: String = format!(
            "{} entries (max {}): {}",
            visible.len(),
            caps::FOLDER_ENTRIES,
            visible.join(", ")
        );
        out.push(violation(root, dir, 0, Rule::FolderEntries, detail));
    }
    if let Some(detail) = underscore_problem(dir, entries, &visible) {
        out.push(violation(root, dir, 0, Rule::Underscore, detail));
    }
}

/// C# trees: at most one `_` entry, and one is required once there are 2+
/// subfolders. Everywhere else (Rust, docs): no `_` entries at all.
fn underscore_problem(dir: &Path, entries: &[PathBuf], visible: &[String]) -> Option<String> {
    let marked: Vec<&String> = visible
        .iter()
        .filter(|n: &&String| n.starts_with('_'))
        .collect();
    if !is_csharp_tree(dir) {
        return marked
            .first()
            .map(|first: &&String| format!("`_` is for C# trees only: {first}"));
    }
    let subdirs: usize = entries
        .iter()
        .filter(|p: &&PathBuf| is_visible_dir(p))
        .count();
    if marked.len() > 1 {
        Some(format!("{} `_` entries, max 1", marked.len()))
    } else if subdirs >= 2 && marked.is_empty() {
        Some("C# folder with 2+ subfolders needs one `_` entry".to_owned())
    } else {
        None
    }
}

fn check_file(root: &Path, file: &Path, out: &mut Vec<Violation>) -> Result<(), AuditError> {
    let ext: String = file
        .extension()
        .and_then(|e: &std::ffi::OsStr| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let cap: Option<usize> = match ext.as_str() {
        "md" => Some(caps::MD_LINES),
        "rs" => Some(caps::RS_LINES),
        "cs" => Some(caps::CS_LINES),
        "toml" | "yaml" | "yml" | "json" | "sh" | "csproj" | "slnx" | "props" => {
            Some(caps::OTHER_LINES)
        }
        _ => None,
    };
    let Some(cap): Option<usize> = cap else {
        return Ok(());
    };
    let text: String =
        std::fs::read_to_string(file).map_err(|source: std::io::Error| AuditError::Read {
            path: file.to_path_buf(),
            source,
        })?;
    let lines: usize = text.lines().count();
    if lines > cap {
        out.push(violation(
            root,
            file,
            0,
            Rule::FileLines,
            format!("{lines} lines (max {cap})"),
        ));
    }
    let findings: Vec<Finding> = match ext.as_str() {
        "rs" => rust::check(&text),
        "cs" => csharp::check(&text),
        _ => Vec::new(),
    };
    for f in findings {
        out.push(violation(root, file, f.line, f.rule, f.detail));
    }
    Ok(())
}

/// A folder is a C# tree when it holds C# project files and nothing Rust.
/// Mixed folders (a Rust repo carrying C# fixtures) belong to neither.
fn is_csharp_tree(dir: &Path) -> bool {
    let mut has_csharp: bool = false;
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(read): std::io::Result<std::fs::ReadDir> = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in read.flatten() {
            let path: PathBuf = entry.path();
            let name: String = file_name(&path);
            if path.is_dir() && !SKIP_DIRS.contains(&name.as_str()) {
                stack.push(path);
            } else if name == "Cargo.toml" || name.ends_with(".rs") {
                return false;
            } else if CSHARP_MARKERS.iter().any(|e: &&str| name.ends_with(e)) {
                has_csharp = true;
            }
        }
    }
    has_csharp
}

fn is_visible_dir(p: &Path) -> bool {
    p.is_dir() && !file_name(p).starts_with('.')
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn violation(root: &Path, path: &Path, line: usize, rule: Rule, detail: String) -> Violation {
    let rel: PathBuf = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    Violation {
        path: rel,
        line,
        rule,
        detail,
    }
}
