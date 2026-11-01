//! The engine's values: names, settings words, results.

mod bb_fqn;
mod results;

pub use bb_fqn::{Fqn, FqnError, FqnParts};
pub use results::{Leaf, Outcome, RunReport, SpecList, SpecResult, Verdict};

/// A build configuration (`Debug`, `Release`, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration(String);

impl Configuration {
    /// A configuration name; `None` when empty or holding white space.
    #[must_use]
    pub fn new(name: &str) -> Option<Self> {
        let ok: bool = !name.is_empty() && !name.chars().any(char::is_whitespace);
        ok.then(|| Self(name.to_owned()))
    }

    /// The Testkit default: `Debug`.
    #[must_use]
    pub fn debug() -> Self {
        Self("Debug".to_owned())
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A test category (NUnit `Category`, xUnit trait `Category`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category(String);

impl Category {
    /// A category name; `None` when empty.
    #[must_use]
    pub fn new(name: &str) -> Option<Self> {
        let name: &str = name.trim();
        (!name.is_empty()).then(|| Self(name.to_owned()))
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
