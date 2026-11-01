//! One registry for every child process group owned by backback.

mod bb_group;

pub use bb_group::{end_all, exit_on_signal, shutdown, GroupId, OwnedGroup};
