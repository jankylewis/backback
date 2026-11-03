//! The file as backback last read or wrote it (BB-BUG-034), and taking
//! the disk's text into a buffer with no unsaved edits (BB-BUG-035).

use std::fs;
use std::path::{Path, PathBuf};

use super::super::{Document, DocumentError};
use crate::{CharEdit, CharIdx};

/// A file's modification time and length.
pub(super) type Stamp = (std::time::SystemTime, u64);

/// The file as last read or written, and whether a save over a newer file
/// was refused once (BB-BUG-034: a save overwrote another tool's change).
#[derive(Debug, Default)]
pub(in crate::document) struct DiskGuard {
    pub(in crate::document) seen: Option<Stamp>,
    pub(in crate::document) armed: bool,
}

pub(super) fn stamp(path: &Path) -> Option<Stamp> {
    let metadata: fs::Metadata = fs::metadata(path).ok()?;
    Some((metadata.modified().ok()?, metadata.len()))
}

impl Document {
    /// Take the disk's text when the file changed on disk and the buffer
    /// holds no unsaved edits, as VS Code does; one undo step, the cursor
    /// kept where it can be. Returns whether the text was taken.
    ///
    /// # Errors
    /// The file cannot be read or is not UTF-8; the buffer is unchanged.
    pub fn reload_if_changed(&mut self) -> Result<bool, DocumentError> {
        let Some(path): Option<PathBuf> = self.path.clone() else {
            return Ok(false);
        };
        let now: Option<Stamp> = stamp(&path);
        if self.is_dirty() || now.is_none() || self.disk.seen.is_none() || now == self.disk.seen {
            return Ok(false);
        }
        let bytes: Vec<u8> = fs::read(&path)?;
        let body: &[u8] = bytes.strip_prefix(&super::BOM).unwrap_or(&bytes);
        let text: String = String::from_utf8(body.to_vec())?;
        let cursor: CharIdx = self.cursor;
        let end: CharIdx = CharIdx::new(self.len_chars());
        self.replace_many(&[CharEdit {
            from: CharIdx::new(0),
            to: end,
            text,
        }])?;
        let kept: CharIdx = CharIdx::new(cursor.raw().min(self.len_chars()));
        self.set_cursor(self.off_crlf(kept))?;
        self.saved_state = self.state;
        self.disk = DiskGuard {
            seen: now,
            armed: false,
        };
        Ok(true)
    }
}
