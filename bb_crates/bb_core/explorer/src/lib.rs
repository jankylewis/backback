//! Lazy folder and solution explorer model.

mod entry;
mod model;

pub use entry::{Entry, EntryKind};
pub use model::{Explorer, ExplorerError};

#[cfg(test)]
mod tests;
