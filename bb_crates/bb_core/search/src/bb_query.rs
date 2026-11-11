//! What a Find in Files query means: `/re/` is a regex, anything else is
//! plain text. Smart case: a query with no capital letter ignores case.
//! [`Options`] are Rider's Cc, W and .* toggles on top of that.

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

/// Find in Files' toggles: match case (else smart case), whole words only,
/// and the whole text as a regex (else only `/re/` is one).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Options {
    /// Cc: case always matters.
    pub case: bool,
    /// W: the match starts and ends at word edges.
    pub word: bool,
    /// .*: the text is a regex.
    pub regex: bool,
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
        Query::with(text, Options::default())
    }

    /// Parses `text` under the toggles in `options`.
    ///
    /// # Errors
    /// As [`Query::parse`].
    pub fn with(text: &str, options: Options) -> Result<Query, SearchError> {
        let slashed: Option<&str> = text
            .strip_prefix('/')
            .and_then(|rest: &str| rest.strip_suffix('/'));
        let body: String = match (options.regex, slashed) {
            (true, _) => text.to_owned(),
            (false, Some(body)) => body.to_owned(),
            (false, None) => regex::escape(text),
        };
        if body.is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        let pattern: String = if options.word {
            format!(r"\b(?:{body})\b")
        } else {
            body
        };
        let ignore_case: bool = !options.case && !text.chars().any(char::is_uppercase);
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
