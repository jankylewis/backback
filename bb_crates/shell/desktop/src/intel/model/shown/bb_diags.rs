//! Diagnostics per open tab, tied to the document version the tab shows.
//!
//! Every `Open` the window sends is version 1 on the server and every
//! `Change` is the next one, in order, so the window knows which version its
//! text is. A list is shown only while its version is that one: squiggles
//! never sit on text they were not computed for, and a late report for an
//! older version never replaces a newer one.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bb_intel::{Diagnostic, DocVersion};

use crate::intel::bb_worker::protocol::Request;

/// A sync request the window sent: each moves the server's version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Sent {
    /// `didOpen`: version 1.
    Open(PathBuf),
    /// One `didChange`: the next version.
    Change(PathBuf),
}

impl Sent {
    /// The version step `request` makes, if any.
    pub(crate) fn of(request: &Request) -> Option<Self> {
        match request {
            Request::Open { path, .. } => Some(Self::Open(path.clone())),
            Request::Change { path, .. } => Some(Self::Change(path.clone())),
            Request::Close { .. } | Request::Ask { .. } => None,
            #[cfg(test)]
            Request::KillForTest => None,
        }
    }
}

/// The versions shown and the newest list per document.
#[derive(Debug, Default)]
pub(crate) struct Diags {
    shown: BTreeMap<PathBuf, DocVersion>,
    lists: BTreeMap<PathBuf, (DocVersion, Vec<Diagnostic>)>,
}

impl Diags {
    /// Follow one sync request the window sent.
    pub(crate) fn sent(&mut self, sent: Sent) {
        match sent {
            Sent::Open(path) => self.opened(&path),
            Sent::Change(path) => self.changed(&path),
        }
    }

    /// The window sent `didOpen` for `path`: its text is version 1.
    fn opened(&mut self, path: &Path) {
        self.shown.insert(path.to_path_buf(), DocVersion::first());
        self.lists.remove(path);
    }

    /// The window sent one `didChange` for `path`. Past the last version
    /// the server refuses the change too, so nothing is shown any more.
    fn changed(&mut self, path: &Path) {
        let next: Option<DocVersion> = self.shown.get(path).and_then(|v: &DocVersion| v.next());
        match next {
            Some(version) => {
                self.shown.insert(path.to_path_buf(), version);
            }
            None => {
                self.shown.remove(path);
            }
        }
    }

    /// The tab closed.
    pub(crate) fn closed(&mut self, path: &Path) {
        self.shown.remove(path);
        self.lists.remove(path);
    }

    /// The server is gone: no list holds any more. The versions restart
    /// with the next `didOpen`.
    pub(crate) fn clear(&mut self) {
        self.shown.clear();
        self.lists.clear();
    }

    /// The version the tab of `path` shows, as the server numbers it.
    pub(crate) fn shown(&self, path: &Path) -> Option<DocVersion> {
        self.shown.get(path).copied()
    }

    /// Keep `list` for `version` unless a newer one is kept. Returns whether
    /// it was kept.
    pub(crate) fn set(
        &mut self,
        path: PathBuf,
        version: DocVersion,
        list: Vec<Diagnostic>,
    ) -> bool {
        let older: bool = self
            .lists
            .get(&path)
            .is_some_and(|(held, _): &(DocVersion, Vec<Diagnostic>)| version < *held);
        if older {
            return false;
        }
        self.lists.insert(path, (version, list));
        true
    }

    /// The list of `path` when it is for the version shown, else nothing.
    pub(crate) fn of(&self, path: &Path) -> &[Diagnostic] {
        match (self.lists.get(path), self.shown.get(path)) {
            (Some((version, list)), Some(shown)) if version == shown => list.as_slice(),
            _ => &[],
        }
    }

    /// Every shown list, by path.
    pub(crate) fn all(&self) -> impl Iterator<Item = (&Path, &[Diagnostic])> {
        self.lists
            .keys()
            .map(|path: &PathBuf| (path.as_path(), self.of(path)))
    }
}
