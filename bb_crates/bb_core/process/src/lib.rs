//! One registry for every child process group owned by backback, and the
//! process environment an app opened from the Dock needs (`shell_env`).

mod bb_group;
pub mod shell_env;

pub use bb_group::{end_all, exit_on_signal, shutdown, GroupId, OwnedGroup};
