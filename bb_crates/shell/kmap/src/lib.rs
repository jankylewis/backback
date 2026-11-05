//! Code map for agents: what each crate, file and item is, read from the
//! source on every call so it can never go stale (`bb map`).
//!
//! Three zoom levels: [`index`] (one line per crate), [`outline`] (each
//! file's purpose and items with line numbers) and [`find`] (where a name is
//! declared). Read the map first, then open only the lines it points to.

mod bb_items;
mod crates;
mod render;

use std::path::PathBuf;

pub use bb_items::{scan, FileMap, Item, Kind, Vis};
pub use crates::{rust_files, workspace, Crate};
pub use render::{find, index, outline};

/// Map errors.
#[derive(Debug, thiserror::Error)]
pub enum KmapError {
    /// A file or folder could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// Path.
        path: PathBuf,
        /// Cause.
        #[source]
        source: std::io::Error,
    },
    /// The folder has no `Cargo.toml` with a `[workspace]` members list.
    #[error("{0} has no Cargo.toml workspace members list (run from bb_crates)")]
    NoWorkspace(PathBuf),
}

/// Read a file as text with a typed error.
pub(crate) fn read(path: &std::path::Path) -> Result<String, KmapError> {
    std::fs::read_to_string(path).map_err(|source: std::io::Error| KmapError::Read {
        path: path.to_path_buf(),
        source,
    })
}
