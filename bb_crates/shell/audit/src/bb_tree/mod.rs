//! Folder rules (HD taxonomy, backbone marker) and file line caps.

mod bb_marker;

use std::path::{Path, PathBuf};

use crate::{caps, csharp, rust, AuditError, Finding, Rule, Violation};
use bb_marker::{Dialect, Dialects};

/// Folders that are never audited: VCS, build output, dependencies.
const SKIP_DIRS: [&str; 6] = [".git", ".target", "target", "bin", "obj", "node_modules"];
/// Agent instruction files: present by law, so outside the HD count.
const AGENT_FILES: [&str; 2] = ["AGENTS.md", "CLAUDE.md"];
/// Generated or legal files that are exempt from line caps.
const SKIP_FILES: [&str; 2] = ["Cargo.lock", "LICENSE"];

pub(crate) fn walk(root: &Path) -> Result<Vec<Violation>, AuditError> {
    let dialects: Dialects = Dialects::scan(root)?;
    let mut out: Vec<Violation> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries: Vec<PathBuf> = read_dir(&dir)?;
        let dialect: Dialect = dialects.of(&dir);
        check_folder(root, &dir, &entries, dialect, &mut out);
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

fn check_folder(
    root: &Path,
    dir: &Path,
    entries: &[PathBuf],
    dialect: Dialect,
    out: &mut Vec<Violation>,
) {
    let visible: Vec<String> = entries
        .iter()
        .map(|p: &PathBuf| file_name(p))
        .filter(|n: &String| !n.starts_with('.'))
        .collect();
    let counted: Vec<&String> = visible
        .iter()
        .filter(|n: &&String| !AGENT_FILES.contains(&n.as_str()))
        .collect();
    if counted.len() > caps::FOLDER_ENTRIES {
        let names: Vec<&str> = counted.iter().map(|n: &&String| n.as_str()).collect();
        let detail: String = format!(
            "{} entries (max {}): {}",
            counted.len(),
            caps::FOLDER_ENTRIES,
            names.join(", ")
        );
        out.push(violation(root, dir, 0, Rule::FolderEntries, detail));
    }
    if let Some(detail) = bb_marker::problem(&visible, dialect) {
        out.push(violation(root, dir, 0, Rule::Marker, detail));
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
        "rs" => rust::check_file(&text, is_test_file(root, file)),
        "cs" => csharp::check(&text),
        _ => Vec::new(),
    };
    for f in findings {
        out.push(violation(root, file, f.line, f.rule, f.detail));
    }
    Ok(())
}

/// A file that holds only test code: under `tests/`, or a `tests.rs` module.
fn is_test_file(root: &Path, file: &Path) -> bool {
    let name: String = file_name(file);
    name == "tests.rs"
        || name == "bb_tests.rs"
        || file
            .strip_prefix(root)
            .unwrap_or(file)
            .components()
            .any(|part: std::path::Component<'_>| part.as_os_str() == "tests")
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
