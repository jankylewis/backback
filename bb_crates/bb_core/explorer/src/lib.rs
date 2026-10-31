//! Lazy folder and solution explorer model.

#[path = "tree/children.rs"]
mod children;
mod entry;
mod ignore_rules;
#[path = "tree/bb_model.rs"]
mod model;

pub use entry::{Entry, EntryKind};
pub use model::{Explorer, ExplorerError};

#[cfg(test)]
mod tests;
