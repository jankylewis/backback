//! One registry for every child process group and temp folder owned by
//! backback, and the process environment an app opened from the Dock needs
//! (`shell_env`).

mod bb_group;
mod owner;
pub mod shell_env;
mod temp;

pub use bb_group::{end_all, exit_on_signal, shutdown, GroupId, OwnedGroup};
pub use temp::{remove_owned, sweep, sweep_all, OwnedTempDir, FAMILIES};
