//! No child group outlives its backback (BB-PRC-009, BBEOD42). Every group
//! [`crate::GroupId::register`] records is watched by the guard
//! ([`bb_guard`]), which ends it however this process ends, and, once
//! [`adopt`] named a folder, written to a record file there, which the next
//! launch sweeps ([`records`]). Who owns a temp folder ([`owner`]) shares
//! the `ps` probes.

mod bb_guard;
mod owner;
pub(crate) mod records;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

pub(crate) use owner::{owner, owns, started};
use records::Record;

/// The groups watched now, each with its leader's start token.
static WATCHED: Mutex<Vec<(u32, String)>> = Mutex::new(Vec::new());
/// This process's own start token, probed once.
static OWN: OnceLock<String> = OnceLock::new();
/// Where this process writes its record, once [`adopt`] named it.
static RECORDS: Mutex<Option<PathBuf>> = Mutex::new(None);

fn watched() -> MutexGuard<'static, Vec<(u32, String)>> {
    WATCHED.lock().unwrap_or_else(PoisonError::into_inner)
}

fn records_dir() -> Option<PathBuf> {
    RECORDS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// At launch, before any other thread: end what earlier backback processes
/// left in `dir` ([`records::sweep`]), record this process's groups there
/// from now on, and start the guard.
///
/// # Errors
/// The folder cannot be made; the guard still starts.
pub fn adopt(dir: &Path) -> std::io::Result<records::Swept> {
    bb_guard::ensure(&watched());
    std::fs::create_dir_all(dir)?;
    let swept: records::Swept = records::sweep(dir);
    *RECORDS.lock().unwrap_or_else(PoisonError::into_inner) = Some(dir.to_path_buf());
    Ok(swept)
}

/// Watch `group`, a leader just spawned.
pub(crate) fn watch(group: u32) {
    let token: String = started(group).unwrap_or_else(|| String::from("-"));
    let mut list: MutexGuard<'static, Vec<(u32, String)>> = watched();
    list.push((group, token.clone()));
    bb_guard::send(&bb_guard::watch_line(group, &token), &list);
    save(&list);
}

/// Stop watching `group`: it ended.
pub(crate) fn forget(group: u32) {
    let mut list: MutexGuard<'static, Vec<(u32, String)>> = watched();
    list.retain(|(watched, _): &(u32, String)| *watched != group);
    bb_guard::send(&bb_guard::forget_line(group), &list);
    save(&list);
}

/// App quit, after every group ended: no record left, the guard exits.
pub(crate) fn close() {
    if let Some(dir) = records_dir() {
        records::remove(&dir, std::process::id());
    }
    bb_guard::close();
}

/// The guard's pid, when one runs.
#[must_use]
pub fn guard_pid() -> Option<u32> {
    bb_guard::pid()
}

fn save(list: &[(u32, String)]) {
    let Some(dir): Option<PathBuf> = records_dir() else {
        return;
    };
    let own: u32 = std::process::id();
    let record: Record = Record {
        owner: (
            own,
            OWN.get_or_init(|| started(own).unwrap_or_else(|| String::from("-")))
                .clone(),
        ),
        groups: list.to_vec(),
    };
    records::write(&dir, &record);
}
