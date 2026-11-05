//! The menu's jobs that run off the UI thread: a delete and a paste.

#[path = "bb_paste.rs"]
mod bb_paste;
mod trash;

use super::{bb_ops, file_name, release, show};
pub(super) use bb_paste::paste;
pub(super) use trash::ask_delete;
#[cfg(test)]
mod tests;
