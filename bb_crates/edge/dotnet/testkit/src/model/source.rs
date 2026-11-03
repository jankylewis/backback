//! Where a test is written: the `.cs` file and line of its method.

use std::path::{Path, PathBuf};

use super::Fqn;

/// Folders a test project's sources never live in.
const SKIPPED: [&str; 4] = ["bin", "obj", "node_modules", "TestResults"];
/// A folder walk stops this deep, and after this many files.
const MAX_DEPTH: usize = 12;
const MAX_FILES: usize = 5000;

/// The file and 0-based line of `fqn`'s method under `dir`: the first line
/// naming the method after the class's declaration, else the declaration.
/// A class declared in several files prefers the one in the test's
/// namespace. `None` when no `.cs` file declares the class.
#[must_use]
pub fn find_source(dir: &Path, fqn: &Fqn) -> Option<(PathBuf, usize)> {
    let parts: super::FqnParts<'_> = fqn.parts();
    let class: &str = simple_class(parts.class);
    let method: &str = parts.spec.split('(').next().unwrap_or(parts.spec);
    let mut files: Vec<PathBuf> = Vec::new();
    collect(dir, 0, &mut files);
    files.sort();
    let mut fallback: Option<(PathBuf, usize)> = None;
    for file in files {
        let Ok(text): Result<String, std::io::Error> = std::fs::read_to_string(&file) else {
            continue;
        };
        let Some(line): Option<usize> = place(&text, class, method) else {
            continue;
        };
        if parts.namespace.is_empty() || declares_namespace(&text, parts.namespace) {
            return Some((file, line));
        }
        fallback.get_or_insert((file, line));
    }
    fallback
}

/// `Outer+Inner` -> `Inner`; a generic arity mark (a backtick and a
/// number) is cut.
fn simple_class(class: &str) -> &str {
    let last: &str = class.rsplit('+').next().unwrap_or(class);
    last.split('`').next().unwrap_or(last)
}

fn collect(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries): Result<std::fs::ReadDir, std::io::Error> = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if files.len() >= MAX_FILES {
            return;
        }
        let path: PathBuf = entry.path();
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Ok(kind): Result<std::fs::FileType, std::io::Error> = entry.file_type() else {
            continue;
        };
        if kind.is_dir() && !name.starts_with('.') && !SKIPPED.contains(&name.as_str()) {
            collect(&path, depth.saturating_add(1), files);
        } else if kind.is_file()
            && path
                .extension()
                .is_some_and(|ext: &std::ffi::OsStr| ext.eq_ignore_ascii_case("cs"))
        {
            files.push(path);
        }
    }
}

fn place(text: &str, class: &str, method: &str) -> Option<usize> {
    let lines: Vec<&str> = text.lines().collect();
    let declared: usize = lines.iter().position(|line: &&str| {
        ["class", "record", "struct"]
            .iter()
            .any(|keyword: &&str| follows(line, keyword, class))
    })?;
    let body: usize = declared.saturating_add(1);
    let named: Option<usize> = lines
        .get(body..)
        .unwrap_or_default()
        .iter()
        .position(|line: &&str| names_method(line, method))
        .map(|at: usize| at.saturating_add(body));
    Some(named.unwrap_or(declared))
}

/// `line` holds `keyword` then `name` as whole words.
fn follows(line: &str, keyword: &str, name: &str) -> bool {
    let words: Vec<&str> = line
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|word: &&str| !word.is_empty())
        .collect();
    words
        .windows(2)
        .any(|pair: &[&str]| pair.first() == Some(&keyword) && pair.get(1) == Some(&name))
}

/// `line` declares or calls `method`: the whole word, then `(` or `<`.
fn names_method(line: &str, method: &str) -> bool {
    line.match_indices(method).any(|(at, _): (usize, &str)| {
        let before: Option<char> = line.get(..at).and_then(|head: &str| head.chars().last());
        let after: Option<char> = line
            .get(at.saturating_add(method.len())..)
            .and_then(|tail: &str| tail.trim_start().chars().next());
        !before.is_some_and(|c: char| c.is_alphanumeric() || c == '_' || c == '.')
            && matches!(after, Some('(' | '<'))
    })
}

fn declares_namespace(text: &str, namespace: &str) -> bool {
    text.lines().any(|line: &str| {
        line.trim()
            .strip_prefix("namespace ")
            .is_some_and(|rest: &str| rest.trim().trim_end_matches([';', '{']).trim() == namespace)
    })
}

#[cfg(test)]
#[path = "source_tests.rs"]
mod tests;
