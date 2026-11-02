//! Find in Files: every line under a root that matches a [`Query`],
//! skipping what Go to File skips (`.git`, `bin`, `obj`, `node_modules`,
//! hidden folders) plus git-ignored, binary and very large files.

mod bb_query;
mod walk;

#[cfg(test)]
mod tests;

pub use bb_query::{Query, SearchError};
pub use walk::{search, Found, Hit};
