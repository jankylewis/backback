//! What a Find in Files query means: `/re/` is a regex, anything else is
//! plain text. Smart case: a query with no capital letter ignores case.

use regex::{Regex, RegexBuilder};

/// Why a query cannot run.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    /// Nothing to search for.
    #[error("the query is empty")]
    EmptyQuery,
    /// `/re/` is not a valid regex.
    #[error("bad regex: {0}")]
    BadPattern(#[from] regex::Error),
    /// A files glob does not parse.
    #[error("bad files glob: {0}")]
    BadGlob(String),
}

/// A parsed query.
#[derive(Debug, Clone)]
pub struct Query {
    matcher: Regex,
}

impl Query {
    /// Parses `text`; see the module rules.
    ///
    /// # Errors
    /// [`SearchError::EmptyQuery`] for an empty query or `//`, and
    /// [`SearchError::BadPattern`] for an invalid regex.
    pub fn parse(text: &str) -> Result<Query, SearchError> {
        let regex: Option<&str> = text
            .strip_prefix('/')
            .and_then(|rest: &str| rest.strip_suffix('/'));
        let pattern: String = match regex {
            Some(body) => body.to_owned(),
            None => regex::escape(text),
        };
        if pattern.is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        let ignore_case: bool = !text.chars().any(char::is_uppercase);
        let matcher: Regex = RegexBuilder::new(&pattern)
            .case_insensitive(ignore_case)
            .multi_line(true)
            .crlf(true)
            .size_limit(1 << 20)
            .build()?;
        Ok(Query { matcher })
    }

    /// Whether `text` (a whole file) may hold a matching line. `^` and `$`
    /// match at every line end, so a matching line is never missed; a match
    /// across lines is only a false alarm the line search rejects.
    #[must_use]
    pub fn may_match(&self, text: &str) -> bool {
        self.matcher.is_match(text)
    }

    /// The byte offset of the first match in `line`.
    #[must_use]
    pub fn find(&self, line: &str) -> Option<usize> {
        self.matcher.find(line).map(|m: regex::Match<'_>| m.start())
    }
}
