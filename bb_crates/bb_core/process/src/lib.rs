//! One registry for every child process group and temp folder owned by
//! backback, the guard and records that end those groups however backback
//! ends (`orphan`), and the process environment an app opened from the Dock
//! needs (`shell_env`).

mod bb_group;
mod orphan;
pub mod shell_env;
mod temp;

pub use bb_group::{end_all, exit_on_signal, shutdown, GroupId, OwnedGroup, GRACE};
pub use orphan::records::{sweep as sweep_records, Swept};
pub use orphan::{adopt, guard_pid};
pub use temp::{remove_owned, sweep, sweep_all, OwnedTempDir, FAMILIES};
