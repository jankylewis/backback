//! Argument parsing. Standard library only; the surface is six commands.

use std::path::PathBuf;

use bb_lsp::roslyn::Restore;

/// Usage text.
pub const USAGE: &str = "usage:
  bb doctor                 check dotnet, roslyn-language-server and netcoredbg
  bb scan <solution>        what each project is: tests, Apex, Playwright, MCP, runsettings
  bb open <solution> [--restore]
                            load a .sln/.slnx through Roslyn and time it;
                            never writes into it unless --restore is given
  bb outline <file.cs>      print the tree-sitter outline of one file
  bb audit [path]           check HD taxonomy, caps, backbone marker, explicit types
  bb bench [solution]       time the fixed benchmark set: table + JSON lines";

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
        /// Whether Roslyn may run `dotnet restore` in the solution.
        restore: Restore,
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
    /// Time the fixed benchmark set.
    Bench {
        /// Extra solution to scan, if any.
        solution: Option<PathBuf>,
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
    /// An argument the command does not take.
    #[error("unexpected argument: {0}")]
    Extra(String),
}

/// Parse the arguments after the program name.
///
/// # Errors
/// [`CliError`] for a missing or unknown command, a missing path, or an
/// argument the command does not take.
pub fn parse(args: &[String]) -> Result<Command, CliError> {
    let mut it: std::slice::Iter<'_, String> = args.iter();
    let word: &String = it.next().ok_or(CliError::Missing)?;
    let path: Option<PathBuf> = it.next().map(PathBuf::from);
    let flag: Option<&String> = it.next();
    if word == "open" && flag.is_some_and(|f: &String| f == "--restore") {
        return open(path, Restore::On, it.next());
    }
    if let Some(extra) = flag {
        return Err(CliError::Extra(extra.clone()));
    }
    match word.as_str() {
        "doctor" => path.map_or(Ok(Command::Doctor), |p: PathBuf| {
            Err(CliError::Extra(p.display().to_string()))
        }),
        "scan" => path
            .map(|solution: PathBuf| Command::Scan { solution })
            .ok_or(CliError::NeedsPath("scan")),
        "open" => open(path, Restore::Off, None),
        "outline" => path
            .map(|file: PathBuf| Command::Outline { file })
            .ok_or(CliError::NeedsPath("outline")),
        "audit" => Ok(Command::Audit {
            root: path.unwrap_or_else(|| PathBuf::from(".")),
        }),
        "bench" => Ok(Command::Bench { solution: path }),
        other => Err(CliError::Unknown(other.to_owned())),
    }
}

fn open(
    path: Option<PathBuf>,
    restore: Restore,
    extra: Option<&String>,
) -> Result<Command, CliError> {
    if let Some(extra) = extra {
        return Err(CliError::Extra(extra.clone()));
    }
    path.map(|solution: PathBuf| Command::Open { solution, restore })
        .ok_or(CliError::NeedsPath("open"))
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
                solution: PathBuf::from("A.slnx"),
                restore: Restore::Off
            })
        );
        assert_eq!(
            parse(&args(&["open", "A.slnx", "--restore"])),
            Ok(Command::Open {
                solution: PathBuf::from("A.slnx"),
                restore: Restore::On
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
    }

    #[test]
    fn parses_audit_and_bench() {
        assert_eq!(
            parse(&args(&["bench"])),
            Ok(Command::Bench { solution: None })
        );
        assert_eq!(
            parse(&args(&["bench", "A.slnx"])),
            Ok(Command::Bench {
                solution: Some(PathBuf::from("A.slnx"))
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

    #[test]
    fn rejects_arguments_a_command_does_not_take() {
        let extra: Result<Command, CliError> = Err(CliError::Extra("x".to_owned()));
        assert_eq!(parse(&args(&["doctor", "x"])), extra);
        assert_eq!(parse(&args(&["scan", "A.slnx", "x"])), extra);
        assert_eq!(parse(&args(&["open", "A.slnx", "x"])), extra);
        assert_eq!(parse(&args(&["open", "A.slnx", "--restore", "x"])), extra);
        assert_eq!(parse(&args(&["outline", "A.cs", "x"])), extra);
        assert_eq!(parse(&args(&["audit", "src", "x"])), extra);
        assert_eq!(parse(&args(&["bench", "A.slnx", "x"])), extra);
    }
}
