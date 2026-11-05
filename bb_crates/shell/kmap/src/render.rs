//! The three views as plain text: crate index, file outline, name lookup.
//! Every line names a path and a line number an agent can open directly.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::bb_items::{scan, FileMap, Item, Kind, Vis};
use crate::crates::{rust_files, workspace, Crate};
use crate::{read, KmapError};

/// Most enum variants printed.
const VARIANTS: usize = 12;

/// One line per crate: name, folder, size, internal uses, description.
///
/// # Errors
/// See [`workspace`].
pub fn index(root: &Path) -> Result<String, KmapError> {
    let crates: Vec<Crate> = workspace(root)?;
    let files: usize = crates.iter().map(|c: &Crate| c.files).sum();
    let lines: usize = crates.iter().map(|c: &Crate| c.lines).sum();
    let mut out: String = format!(
        "{} crates, {files} files, {lines} lines. Zoom in: bb map <path> | bb map --find <name>\n",
        crates.len()
    );
    for c in &crates {
        let uses: String = if c.uses.is_empty() {
            "-".to_owned()
        } else {
            c.uses.join(" ")
        };
        let _infallible: std::fmt::Result = writeln!(
            out,
            "{} {} ({} files, {} lines) uses: {uses}\n  {}",
            c.name,
            shown(&c.path).display(),
            c.files,
            c.lines,
            c.description.as_deref().unwrap_or("")
        );
    }
    Ok(out)
}

/// Each file under `path`: lines, purpose, then its items (public ones only
/// unless `all`). Test files show only their test count.
///
/// # Errors
/// [`KmapError::Read`] when a file or folder cannot be read.
pub fn outline(path: &Path, all: bool) -> Result<String, KmapError> {
    let mut out: String = String::new();
    for file in rust_files(path)? {
        let map: FileMap = scan(&read(&file)?);
        let purpose: &str = map.purpose.as_deref().unwrap_or("");
        let _infallible: std::fmt::Result = writeln!(
            out,
            "{} ({} lines) {purpose}",
            shown(&file).display(),
            map.lines
        );
        if is_test_file(&file) {
            let _infallible: std::fmt::Result = writeln!(out, "  tests: {}", map.tests);
            continue;
        }
        for (at, item) in map.items.iter().enumerate() {
            if visible(&map.items, at, all) {
                out.push_str(&entry(item));
            }
        }
        if map.tests > 0 {
            let _infallible: std::fmt::Result = writeln!(out, "  tests: {}", map.tests);
        }
    }
    Ok(out)
}

/// Where `name` (or `Owner::name`) is declared, any visibility, under `root`.
///
/// # Errors
/// [`KmapError::Read`] when a file or folder cannot be read.
pub fn find(root: &Path, name: &str) -> Result<String, KmapError> {
    let mut out: String = String::new();
    for file in rust_files(root)? {
        for item in scan(&read(&file)?).items {
            let owned: Option<String> = item
                .owner
                .as_ref()
                .map(|o: &String| format!("{o}::{}", item.name));
            if item.name == name || owned.as_deref() == Some(name) {
                let _infallible: std::fmt::Result = writeln!(
                    out,
                    "{}:{} {}{}",
                    shown(&file).display(),
                    item.line,
                    item.signature,
                    doc(&item)
                );
            }
        }
    }
    if out.is_empty() {
        let _infallible: std::fmt::Result = writeln!(out, "no item named {name}");
    }
    Ok(out)
}

fn visible(items: &[Item], at: usize, all: bool) -> bool {
    let Some(item): Option<&Item> = items.get(at) else {
        return false;
    };
    if all {
        return true;
    }
    if item.kind != Kind::Impl {
        return item.vis != Vis::Private;
    }
    let after: &[Item] = items.get(at.saturating_add(1)..).unwrap_or(&[]);
    item.signature.contains(" for ")
        || after
            .iter()
            .take_while(|m: &&Item| m.owner.is_some() && m.kind != Kind::Impl)
            .any(|m: &Item| m.vis != Vis::Private)
}

fn entry(item: &Item) -> String {
    let indent: &str = if item.owner.is_some() { "    " } else { "  " };
    let mut variants: String = String::new();
    if !item.variants.is_empty() {
        let mut shown: Vec<&str> = item
            .variants
            .iter()
            .take(VARIANTS)
            .map(String::as_str)
            .collect();
        if item.variants.len() > VARIANTS {
            shown.push("…");
        }
        variants = format!(" {{ {} }}", shown.join(", "));
    }
    format!(
        "{indent}{} {}{variants}{}\n",
        item.line,
        item.signature,
        doc(item)
    )
}

fn doc(item: &Item) -> String {
    item.doc
        .as_ref()
        .map_or_else(String::new, |d: &String| format!("  // {d}"))
}

fn is_test_file(file: &Path) -> bool {
    let name: &str = file
        .file_name()
        .and_then(|n: &std::ffi::OsStr| n.to_str())
        .unwrap_or("");
    name == "tests.rs"
        || name.ends_with("_tests.rs")
        || file
            .components()
            .any(|c: std::path::Component<'_>| c.as_os_str() == "tests")
}

fn shown(path: &Path) -> PathBuf {
    path.strip_prefix(".").unwrap_or(path).to_path_buf()
}
