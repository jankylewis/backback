//! The language servers next to Roslyn, each started the same way
//! (`start`): rust-analyzer for Rust (BBEOD27) and pyright for Python
//! (BBEOD28).

pub(crate) mod python;
pub(crate) mod rust;
#[path = "bb_start.rs"]
pub(crate) mod start;
