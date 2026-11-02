//! Word moves and word deletes (`Option+arrows`, `Option+Backspace/Delete`),
//! BB-BUG-011. Stops follow VS Code / `IntelliJ`: the end (start) of the next
//! (previous) word or punctuation run; a line break is a stop of its own,
//! and an indent that runs into a line edge stops at that edge.

use super::super::{Document, DocumentError};
use crate::CharIdx;

/// What a char counts as for word stops.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Space,
    Break,
    Word,
    Punct,
}

fn class(ch: char) -> Class {
    if ch == '\r' || ch == '\n' {
        Class::Break
    } else if ch == ' ' || ch == '\t' {
        Class::Space
    } else if ch.is_alphanumeric() || ch == '_' {
        Class::Word
    } else {
        Class::Punct
    }
}

impl Document {
    /// Option+Right: to the end of the next word or punctuation run.
    ///
    /// # Errors
    /// Returns a checked selection error.
    pub fn word_right(&mut self, extend: bool) -> Result<(), DocumentError> {
        let target: usize = self.word_end(self.cursor.raw());
        self.move_to(CharIdx::new(target), extend)
    }

    /// Option+Left: to the start of the previous word or punctuation run.
    ///
    /// # Errors
    /// Returns a checked selection error.
    pub fn word_left(&mut self, extend: bool) -> Result<(), DocumentError> {
        let target: usize = self.word_start(self.cursor.raw());
        self.move_to(CharIdx::new(target), extend)
    }

    /// Option+Backspace: the selection, or back to the previous word stop.
    ///
    /// # Errors
    /// Returns a checked text error.
    pub fn delete_word_back(&mut self) -> Result<(), DocumentError> {
        if self.selection().is_some() {
            return self.backspace();
        }
        let start: usize = self.word_start(self.cursor.raw());
        self.delete_range(CharIdx::new(start), self.cursor)
    }

    /// Option+Delete: the selection, or on to the next word stop.
    ///
    /// # Errors
    /// Returns a checked text error.
    pub fn delete_word_forward(&mut self) -> Result<(), DocumentError> {
        if self.selection().is_some() {
            return self.delete();
        }
        let end: usize = self.word_end(self.cursor.raw());
        self.delete_range(self.cursor, CharIdx::new(end))
    }

    fn class_at(&self, at: usize) -> Option<Class> {
        self.rope().get_char(at).map(class)
    }

    fn class_before(&self, at: usize) -> Option<Class> {
        at.checked_sub(1)
            .and_then(|before: usize| self.class_at(before))
    }

    fn word_end(&self, from: usize) -> usize {
        let mut at: usize = from;
        match self.class_at(at) {
            None => return at,
            Some(Class::Break) => {
                at = at.saturating_add(1);
                if self.mid_crlf(at) {
                    at = at.saturating_add(1);
                }
                return at;
            }
            Some(Class::Space | Class::Word | Class::Punct) => {}
        }
        while self.class_at(at) == Some(Class::Space) {
            at = at.saturating_add(1);
        }
        let run: Class = match self.class_at(at) {
            None | Some(Class::Break) => return at,
            Some(found @ (Class::Space | Class::Word | Class::Punct)) => found,
        };
        while self.class_at(at) == Some(run) {
            at = at.saturating_add(1);
        }
        at
    }

    fn word_start(&self, from: usize) -> usize {
        let mut at: usize = from;
        match self.class_before(at) {
            None => return at,
            Some(Class::Break) => {
                at = at.saturating_sub(1);
                if self.mid_crlf(at) {
                    at = at.saturating_sub(1);
                }
                return at;
            }
            Some(Class::Space | Class::Word | Class::Punct) => {}
        }
        while self.class_before(at) == Some(Class::Space) {
            at = at.saturating_sub(1);
        }
        let run: Class = match self.class_before(at) {
            None | Some(Class::Break) => return at,
            Some(found @ (Class::Space | Class::Word | Class::Punct)) => found,
        };
        while self.class_before(at) == Some(run) {
            at = at.saturating_sub(1);
        }
        at
    }
}

#[cfg(test)]
#[path = "words_tests.rs"]
mod tests;
