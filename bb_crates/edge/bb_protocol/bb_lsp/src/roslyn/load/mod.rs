//! Waiting for Roslyn to load a solution: the load log's state and the
//! wait for every project.

mod bb_load;
mod load_state;
#[cfg(test)]
mod load_tests;

pub use bb_load::{
    drain_log, is_project_reload, log_incoming, wait_for_load, wait_for_restore, Loaded,
};
pub use load_state::LoadState;
