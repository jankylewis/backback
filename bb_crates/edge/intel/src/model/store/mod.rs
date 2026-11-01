//! Diagnostics per document, each tied to the document version it was
//! computed for. A report for an older version never replaces a newer one,
//! and a poisoned lock is an error, never an empty list.

#[cfg(test)]
mod bb_tests;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::model::bb_doc::DocVersion;
use crate::model::DocPath;
use crate::IntelError;

/// One stored report.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Entry {
    /// The version the report is for; `None` when the server did not say.
    pub(crate) version: Option<DocVersion>,
    /// The diagnostics as the server sent them.
    pub(crate) list: Vec<lsp_types::Diagnostic>,
}

/// What [`Store::record`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub(crate) enum Recorded {
    /// The report is now the stored one.
    Kept,
    /// A newer report is stored; this one was dropped.
    Stale,
}

/// Shared by the session and its notification pump.
#[derive(Clone, Debug, Default)]
pub(crate) struct Store(Arc<Mutex<HashMap<DocPath, Entry>>>);

type Map = HashMap<DocPath, Entry>;

/// Whether a report for `incoming` may replace one for `stored`. A report
/// that names no version cannot prove it is newer than one that does.
pub(crate) fn accepts(stored: Option<DocVersion>, incoming: Option<DocVersion>) -> bool {
    match (stored, incoming) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(held), Some(new)) => new >= held,
    }
}

impl Store {
    fn locked(&self) -> Result<MutexGuard<'_, Map>, IntelError> {
        self.0
            .lock()
            .map_err(|_: PoisonError<MutexGuard<'_, Map>>| IntelError::StorePoisoned)
    }

    /// Keep `entry` for `path` unless a newer report is stored.
    ///
    /// # Errors
    /// [`IntelError::StorePoisoned`].
    pub(crate) fn record(&self, path: &DocPath, entry: Entry) -> Result<Recorded, IntelError> {
        let mut map: MutexGuard<'_, Map> = self.locked()?;
        let stored: Option<DocVersion> = map.get(path).and_then(|e: &Entry| e.version);
        if !accepts(stored, entry.version) {
            return Ok(Recorded::Stale);
        }
        map.insert(path.clone(), entry);
        Ok(Recorded::Kept)
    }

    /// The stored report for `path`, if any.
    ///
    /// # Errors
    /// [`IntelError::StorePoisoned`].
    pub(crate) fn entry(&self, path: &DocPath) -> Result<Option<Entry>, IntelError> {
        Ok(self.locked()?.get(path).cloned())
    }

    /// Drop the report for `path`.
    ///
    /// # Errors
    /// [`IntelError::StorePoisoned`].
    pub(crate) fn remove(&self, path: &DocPath) -> Result<(), IntelError> {
        self.locked()?.remove(path);
        Ok(())
    }
}
