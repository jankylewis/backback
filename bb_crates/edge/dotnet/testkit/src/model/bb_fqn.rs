//! Fully qualified test names and the tree levels they split into.

use std::fmt;

/// Why a text is not a fully qualified test name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FqnError {
    /// Empty, or only white space.
    #[error("empty test name")]
    Empty,
    /// No `Class.Method` part before the arguments.
    #[error("test name has no class: {0}")]
    NoClass(String),
}

/// A fully qualified test name: `Namespace.Class.Method(args)`, the
/// namespace and arguments optional. The arguments may hold dots.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fqn(String);

/// The tree levels of one [`Fqn`]: Namespace > Class > Spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FqnParts<'a> {
    /// Everything before the class; empty for the global namespace.
    pub namespace: &'a str,
    /// The class (nested classes keep their `+`).
    pub class: &'a str,
    /// The method, with its arguments when parameterised.
    pub spec: &'a str,
}

impl Fqn {
    /// Check and keep `raw` (surrounding white space removed).
    ///
    /// # Errors
    /// [`FqnError`] when it is empty or has no `Class.Method` part.
    pub fn new(raw: &str) -> Result<Self, FqnError> {
        let text: &str = raw.trim();
        if text.is_empty() {
            return Err(FqnError::Empty);
        }
        let fqn: Self = Self(text.to_owned());
        let parts: Option<FqnParts<'_>> = fqn.split();
        if parts.is_none_or(|p: FqnParts<'_>| p.class.is_empty() || p.spec.is_empty()) {
            return Err(FqnError::NoClass(text.to_owned()));
        }
        Ok(fqn)
    }

    /// The name as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The name without arguments: `Namespace.Class.Method`.
    #[must_use]
    pub fn method(&self) -> &str {
        self.0
            .split_once('(')
            .map_or(self.0.as_str(), |(head, _): (&str, &str)| head)
    }

    /// Namespace, class and spec. Never `None` for a checked name.
    #[must_use]
    pub fn parts(&self) -> FqnParts<'_> {
        self.split().unwrap_or(FqnParts {
            namespace: "",
            class: "",
            spec: &self.0,
        })
    }

    fn split(&self) -> Option<FqnParts<'_>> {
        let head_len: usize = self.method().len();
        let head: &str = self.0.get(..head_len)?;
        let dot: usize = head.rfind('.')?;
        let spec: &str = self.0.get(dot.checked_add(1)?..)?;
        let owner: &str = head.get(..dot)?;
        let (namespace, class): (&str, &str) = match owner.rfind('.') {
            Some(cut) => (owner.get(..cut)?, owner.get(cut.checked_add(1)?..)?),
            None => ("", owner),
        };
        Some(FqnParts {
            namespace,
            class,
            spec,
        })
    }
}

impl fmt::Display for Fqn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
