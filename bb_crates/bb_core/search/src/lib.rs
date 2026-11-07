//! Find in Files: every line under a root that matches a [`Query`],
//! skipping what Go to File skips (`.git`, `bin`, `obj`, `node_modules`,
//! VCS and IDE caches) plus git-ignored, binary and very large files;
//! other dot-folders are searched (BBEOD21).

mod bb_query;
mod filter;
mod walk;

#[cfg(test)]
mod tests;

pub use bb_query::{Query, SearchError};
pub use filter::Filter;
pub use walk::{files, search, search_in, Found, Hit};
