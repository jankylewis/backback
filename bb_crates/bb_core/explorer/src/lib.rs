//! Lazy folder and solution explorer model.

mod bb_entry;
#[path = "tree/children.rs"]
mod children;
mod ignore_rules;
#[path = "tree/bb_model.rs"]
mod model;

pub use bb_entry::{Entry, EntryKind};
pub use model::{Explorer, ExplorerError};

#[cfg(test)]
mod tests;
