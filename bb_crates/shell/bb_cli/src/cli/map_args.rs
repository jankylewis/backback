//! `bb map [path] [--all]` and `bb map --find <name> [path]`.

use std::path::PathBuf;

use super::CliError;

/// What `bb map` shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapArgs {
    /// One line per crate of the workspace in the current folder.
    Index,
    /// Files and items under a path; private items too with `--all`.
    Outline {
        /// File or folder.
        path: PathBuf,
        /// Include private items.
        all: bool,
    },
    /// Where a name is declared.
    Find {
        /// Item name, or `Owner::name`.
        name: String,
        /// Folder to search (default: current folder).
        root: PathBuf,
    },
}

/// Parse the words after `map`.
///
/// # Errors
/// [`CliError::NeedsPath`] for `--find` without a name, [`CliError::Extra`]
/// for any other extra word.
pub fn parse(rest: &[String]) -> Result<MapArgs, CliError> {
    let words: Vec<&str> = rest.iter().map(String::as_str).collect();
    match words.as_slice() {
        [] => Ok(MapArgs::Index),
        ["--find"] => Err(CliError::NeedsPath("map --find")),
        ["--find", name] => Ok(find(name, ".")),
        ["--find", name, root] => Ok(find(name, root)),
        ["--all"] => Ok(outline(".", true)),
        [path] => Ok(outline(path, false)),
        [path, "--all"] | ["--all", path] => Ok(outline(path, true)),
        [.., last] => Err(CliError::Extra((*last).to_owned())),
    }
}

fn find(name: &str, root: &str) -> MapArgs {
    MapArgs::Find {
        name: name.to_owned(),
        root: PathBuf::from(root),
    }
}

fn outline(path: &str, all: bool) -> MapArgs {
    MapArgs::Outline {
        path: PathBuf::from(path),
        all,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn map_words_pick_the_view() {
        assert_eq!(parse(&words("")), Ok(MapArgs::Index));
        assert_eq!(parse(&words("edge --all")), Ok(outline("edge", true)));
        assert_eq!(parse(&words("edge")), Ok(outline("edge", false)));
        assert_eq!(parse(&words("--find scan")), Ok(find("scan", ".")));
        assert_eq!(parse(&words("--find scan edge")), Ok(find("scan", "edge")));
        assert_eq!(
            parse(&words("--find")),
            Err(CliError::NeedsPath("map --find"))
        );
        assert_eq!(parse(&words("a b")), Err(CliError::Extra("b".to_owned())));
    }
}
