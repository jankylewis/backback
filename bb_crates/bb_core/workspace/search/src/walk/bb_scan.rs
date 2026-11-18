//! One batch of files read and searched across the cores (BB-PRF-006);
//! each file's hits, in the batch's order.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::Hit;
use crate::Query;

/// A preview keeps at most this many chars.
const PREVIEW_CHARS: usize = 200;
/// Larger files are skipped: generated or data, not source.
const MAX_BYTES: u64 = 2 * 1024 * 1024;

/// Files read and searched at once: enough to keep every core busy, few
/// enough that a reached cap stops soon.
pub(super) const BATCH: usize = 128;
/// Threads at most; more only queue on the disk.
pub(super) const MAX_WORKERS: usize = 8;

/// Each file of `batch`, in order: its hits, or `None` when it is not read
/// (big, binary, unreadable, or the search was stopped). The batch is split
/// into one run of files per worker.
pub(super) fn scan(
    batch: &[PathBuf],
    query: &Query,
    workers: usize,
    stop: &AtomicBool,
) -> Vec<Option<Vec<Hit>>> {
    let size: usize = batch.len().div_ceil(workers.max(1)).max(1);
    std::thread::scope(|scope: &std::thread::Scope<'_, '_>| {
        let parts: Vec<std::thread::ScopedJoinHandle<'_, Vec<Option<Vec<Hit>>>>> = batch
            .chunks(size)
            .map(|part: &[PathBuf]| {
                scope.spawn(move || {
                    part.iter()
                        .map(|path: &PathBuf| {
                            (!stop.load(Ordering::Relaxed))
                                .then(|| file_hits(path, query))
                                .flatten()
                        })
                        .collect()
                })
            })
            .collect();
        parts
            .into_iter()
            .flat_map(
                |part: std::thread::ScopedJoinHandle<'_, Vec<Option<Vec<Hit>>>>| {
                    part.join().unwrap_or_default()
                },
            )
            .collect()
    })
}

/// The hits of one file; `None` when it is not read as text.
fn file_hits(path: &Path, query: &Query) -> Option<Vec<Hit>> {
    let text: String = read_text(path)?;
    if !query.may_match(&text) {
        return Some(Vec::new());
    }
    Some(
        text.lines()
            .enumerate()
            .filter_map(|(index, line): (usize, &str)| {
                query
                    .find(line)
                    .map(|start: usize| hit(path, index, line, start))
            })
            .collect(),
    )
}

/// The file as UTF-8 text; `None` for big, binary or unreadable files.
fn read_text(path: &Path) -> Option<String> {
    let size: u64 = std::fs::metadata(path).ok()?.len();
    if size > MAX_BYTES {
        return None;
    }
    let bytes: Vec<u8> = std::fs::read(path).ok()?;
    if bytes.iter().take(8 * 1024).any(|b: &u8| *b == 0) {
        return None;
    }
    String::from_utf8(bytes).ok()
}

fn hit(path: &Path, index: usize, line: &str, start: usize) -> Hit {
    let before: &str = line.get(..start).unwrap_or_default();
    let units: usize = before.encode_utf16().count();
    Hit {
        path: path.to_path_buf(),
        line: u32::try_from(index).unwrap_or(u32::MAX),
        column: u32::try_from(units).unwrap_or(u32::MAX),
        preview: preview(line),
    }
}

fn preview(line: &str) -> String {
    let trimmed: &str = line.trim();
    if trimmed.chars().count() <= PREVIEW_CHARS {
        return trimmed.to_owned();
    }
    let mut short: String = trimmed.chars().take(PREVIEW_CHARS).collect();
    short.push('…');
    short
}
