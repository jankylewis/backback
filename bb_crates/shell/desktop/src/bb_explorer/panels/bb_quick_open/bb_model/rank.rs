//! The box's labels: how a place reads, and how well a label matches a
//! typed query.

use std::path::Path;

/// 0: the query is in the file name; 1: in the path; None: no match.
pub(super) fn score(file: &Path, query: &str) -> Option<u8> {
    let name: String = file.file_name()?.to_string_lossy().to_lowercase();
    if subsequence(&name, query) {
        return Some(0);
    }
    subsequence(&file.to_string_lossy().to_lowercase(), query).then_some(1)
}

fn subsequence(text: &str, query: &str) -> bool {
    let mut chars: std::str::Chars<'_> = text.chars();
    query
        .chars()
        .all(|wanted: char| chars.any(|ch: char| ch == wanted))
}

pub(super) fn project_label(dir: &Path, root: &Path) -> String {
    format!("Project · {} · {}", name_of(dir), shown(dir, root))
}

pub(super) fn name_of(dir: &Path) -> String {
    dir.file_name().map_or_else(
        || dir.display().to_string(),
        |n: &std::ffi::OsStr| n.to_string_lossy().into_owned(),
    )
}

/// `dir` under `root`, or `.` for the root itself.
pub(super) fn shown(dir: &Path, root: &Path) -> String {
    match dir.strip_prefix(root) {
        Ok(rest) if rest.as_os_str().is_empty() => String::from("."),
        Ok(rest) => rest.display().to_string(),
        Err(_) => dir.display().to_string(),
    }
}
