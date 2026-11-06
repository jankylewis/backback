//! How well a label matches a typed query.

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
