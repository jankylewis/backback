//! Argument parsing. Standard library only; the surface is five commands.

use std::path::PathBuf;

/// Usage text.
pub const USAGE: &str = "usage:
  bb doctor                 check dotnet, roslyn-language-server and netcoredbg
  bb scan <solution>        what each project is: tests, Apex, Playwright, MCP, runsettings
  bb open <solution>        load a .sln/.slnx through Roslyn and time it
  bb outline <file.cs>      print the tree-sitter outline of one file
  bb audit [path]           check HD taxonomy, caps, underscore rule, explicit types";

/// A parsed command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Check the toolchain.
    Doctor,
    /// Describe every project of a solution.
    Scan {
        /// Solution path.
        solution: PathBuf,
    },
    /// Load a solution through Roslyn.
    Open {
        /// Solution path.
        solution: PathBuf,
    },
    /// Outline a C# file.
    Outline {
        /// File path.
        file: PathBuf,
    },
    /// Audit conventions under a folder.
    Audit {
        /// Folder to audit (default: current folder).
        root: PathBuf,
    },
}

/// Parse errors.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CliError {
    /// No command given.
    #[error("no command")]
    Missing,
    /// Unknown command word.
    #[error("unknown command: {0}")]
    Unknown(String),
    /// A command needed a path.
    #[error("{0} needs a path argument")]
    NeedsPath(&'static str),
}

/// Parse the arguments after the program name.
///
/// # Errors
/// [`CliError`] for a missing or unknown command, or a missing path.
pub fn parse(args: &[String]) -> Result<Command, CliError> {
    let mut it: std::slice::Iter<'_, String> = args.iter();
    let word: &String = it.next().ok_or(CliError::Missing)?;
    let path: Option<PathBuf> = it.next().map(PathBuf::from);
    match word.as_str() {
        "doctor" => Ok(Command::Doctor),
        "scan" => path
            .map(|solution: PathBuf| Command::Scan { solution })
            .ok_or(CliError::NeedsPath("scan")),
        "open" => path
            .map(|solution: PathBuf| Command::Open { solution })
            .ok_or(CliError::NeedsPath("open")),
        "outline" => path
            .map(|file: PathBuf| Command::Outline { file })
            .ok_or(CliError::NeedsPath("outline")),
        "audit" => Ok(Command::Audit {
            root: path.unwrap_or_else(|| PathBuf::from(".")),
        }),
        other => Err(CliError::Unknown(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s: &&str| (*s).to_owned()).collect()
    }

    #[test]
    fn parses_each_command() {
        assert_eq!(parse(&args(&["doctor"])), Ok(Command::Doctor));
        assert_eq!(
            parse(&args(&["scan", "A.slnx"])),
            Ok(Command::Scan {
                solution: PathBuf::from("A.slnx")
            })
        );
        assert_eq!(
            parse(&args(&["open", "A.slnx"])),
            Ok(Command::Open {
                solution: PathBuf::from("A.slnx")
            })
        );
        assert_eq!(
            parse(&args(&["outline", "A.cs"])),
            Ok(Command::Outline {
                file: PathBuf::from("A.cs")
            })
        );
        assert_eq!(
            parse(&args(&["audit"])),
            Ok(Command::Audit {
                root: PathBuf::from(".")
            })
        );
        assert_eq!(
            parse(&args(&["audit", "src"])),
            Ok(Command::Audit {
                root: PathBuf::from("src")
            })
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(parse(&args(&["open"])), Err(CliError::NeedsPath("open")));
        assert_eq!(parse(&args(&["scan"])), Err(CliError::NeedsPath("scan")));
        assert_eq!(parse(&args(&[])), Err(CliError::Missing));
        assert_eq!(
            parse(&args(&["fly"])),
            Err(CliError::Unknown("fly".to_owned()))
        );
    }
}
