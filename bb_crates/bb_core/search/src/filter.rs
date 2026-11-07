//! Which files a search reads, as VS Code's "files to include / exclude"
//! and ripgrep's `-g`: comma-separated globs; `!` before one excludes it.
//! A plain name with no wildcard (`Tests`, `!obj`) also covers everything
//! under a folder of that name.

use std::path::Path;

use ignore::overrides::{Override, OverrideBuilder};

use crate::SearchError;

/// The files field of Find in Files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    globs: Vec<String>,
}

impl Filter {
    /// Parses `text`; an empty text keeps every file.
    ///
    /// # Errors
    /// A glob that does not parse.
    pub fn parse(text: &str) -> Result<Filter, SearchError> {
        let mut globs: Vec<String> = Vec::new();
        for item in text
            .split(',')
            .map(str::trim)
            .filter(|i: &&str| !i.is_empty())
        {
            let (bang, name): (&str, &str) = match item.strip_prefix('!') {
                Some(rest) => ("!", rest.trim()),
                None => ("", item),
            };
            globs.push(format!("{bang}{name}"));
            if !name.contains(['*', '?', '[']) {
                globs.push(format!("{bang}**/{}/**", name.trim_matches('/')));
            }
        }
        let filter: Filter = Filter { globs };
        filter
            .overrides(Path::new("/"))
            .map_err(|error: ignore::Error| SearchError::BadGlob(error.to_string()))?;
        Ok(filter)
    }

    /// True when every file is kept.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.globs.is_empty()
    }

    /// The globs as the walk's overrides under `root`.
    pub(crate) fn overrides(&self, root: &Path) -> Result<Override, ignore::Error> {
        let mut builder: OverrideBuilder = OverrideBuilder::new(root);
        for glob in &self.globs {
            let _added: &mut OverrideBuilder = builder.add(glob)?;
        }
        builder.build()
    }
}
