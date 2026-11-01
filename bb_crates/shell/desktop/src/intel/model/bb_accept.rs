//! The one guard against stale answers. A question records the document's
//! revision; its answer is used only if it is still the newest question of
//! its kind and the text still fits the kind's rule.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use bb_text::{CharIdx, LspPoint};

use crate::intel::bb_worker::protocol::{AskKind, Ticket};

/// One question in flight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pending {
    /// Its ticket.
    pub(crate) ticket: Ticket,
    /// Its kind.
    pub(crate) kind: AskKind,
    /// The document asked about.
    pub(crate) path: PathBuf,
    /// The document's revision when asked.
    pub(crate) revision: u64,
    /// Where it was asked.
    pub(crate) at: LspPoint,
    /// The start of the word being completed (completion only).
    pub(crate) anchor: CharIdx,
    /// When the UI queued it.
    pub(crate) sent: Instant,
}

/// How much the text may have changed before an answer is useless.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Freshness {
    /// The text must be the one asked about (positions inside the answer).
    SameText,
    /// The cursor must still be in the word that was being completed.
    SameWord,
    /// Any text: the answer points at other places that stay meaningful.
    Any,
}

/// The freshness rule of each kind. Exhaustive: a new kind must pick one.
pub(crate) fn freshness(kind: AskKind) -> Freshness {
    match kind {
        AskKind::Hover | AskKind::Rename | AskKind::Actions => Freshness::SameText,
        AskKind::Completion => Freshness::SameWord,
        AskKind::Definition | AskKind::References => Freshness::Any,
    }
}

/// The editor as an answer arrives, seen from the question's document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Now {
    /// The active tab's document.
    pub(crate) active: Option<PathBuf>,
    /// The revision of the question's document, if it is still open.
    pub(crate) revision: Option<u64>,
    /// Where the word at the cursor of the active tab starts.
    pub(crate) word_start: Option<CharIdx>,
}

/// Whether an answer to `pending` still fits the editor under `rule`.
pub(crate) fn fits(pending: &Pending, rule: Freshness, now: &Now) -> bool {
    match rule {
        Freshness::SameText => now.revision == Some(pending.revision),
        Freshness::SameWord => {
            now.active.as_deref() == Some(pending.path.as_path())
                && now.word_start == Some(pending.anchor)
        }
        Freshness::Any => true,
    }
}

/// The questions in flight: one slot per kind, newest wins.
#[derive(Debug, Default)]
pub(crate) struct InFlight {
    next: u64,
    slots: BTreeMap<AskKind, Pending>,
}

/// What [`InFlight::ask`] needs to know about the question.
pub(crate) struct Question {
    pub(crate) kind: AskKind,
    pub(crate) path: PathBuf,
    pub(crate) revision: u64,
    pub(crate) at: LspPoint,
    pub(crate) anchor: CharIdx,
}

impl InFlight {
    /// Fail every pending question after the server connection closes.
    pub(crate) fn clear(&mut self) {
        self.slots.clear();
    }
    /// Record a new question; any older one of the same kind is forgotten.
    pub(crate) fn ask(&mut self, question: Question) -> Ticket {
        self.next = self.next.saturating_add(1);
        let ticket: Ticket = Ticket(self.next);
        self.slots.insert(
            question.kind,
            Pending {
                ticket,
                kind: question.kind,
                path: question.path,
                revision: question.revision,
                at: question.at,
                anchor: question.anchor,
                sent: Instant::now(),
            },
        );
        ticket
    }

    /// Take the question `ticket` answers, if it is still the newest of its
    /// kind and it [`fits`] the editor `now` under its kind's rule.
    pub(crate) fn accept(&mut self, ticket: Ticket, now: &Now) -> Option<Pending> {
        let kind: AskKind = self
            .slots
            .values()
            .find(|p: &&Pending| p.ticket == ticket)
            .map(|p: &Pending| p.kind)?;
        let pending: Pending = self.slots.remove(&kind)?;
        fits(&pending, freshness(kind), now).then_some(pending)
    }

    /// The question `ticket` answers, if it is still the newest of its kind.
    pub(crate) fn peek(&self, ticket: Ticket) -> Option<&Pending> {
        self.slots.values().find(|p: &&Pending| p.ticket == ticket)
    }

    /// Forget every question about `path` (its tab closed).
    pub(crate) fn forget(&mut self, path: &std::path::Path) {
        self.slots
            .retain(|_: &AskKind, p: &mut Pending| p.path != path);
    }

    /// Whether no question waits for its answer.
    pub(crate) fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Whether a question of `kind` waits for its answer.
    #[cfg(test)]
    pub(crate) fn waiting(&self, kind: AskKind) -> bool {
        self.slots.contains_key(&kind)
    }
}
