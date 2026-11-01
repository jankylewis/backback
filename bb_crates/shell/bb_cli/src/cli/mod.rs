//! Argument parsing. Standard library only; the surface is nine commands.

mod bb_probe;
mod test_args;

pub use bb_probe::{Feature, Probe, MAX_REPEAT};
pub use test_args::{BackendChoice, TestAction, TestArgs};

use std::path::PathBuf;

mod mcp;
pub use mcp::McpCommand;

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
  bb bench [solution] [--runs N]
                            time the fixed set: 1 warm-up + N runs (default 5)
  bb probe <file.cs> <line:col> <feature> [--sln <solution>] [--repeat N]
                            ask Roslyn one feature at a 1-based line:col (restore off);
                            feature: completion, hover, definition, references,
                            rename=<NewName>, actions, diagnostics
  bb test list <solution|project> [--backend apex|dotnet] [--apex-command CMD] [-c CONFIG]
                            list specs as Project > Namespace > Class > Spec (no build)
  bb test run <project> [--specs A,B] [--category X] [--settings FILE] [--build]
                            [--backend apex|dotnet] [--apex-command CMD] [-c CONFIG]
                            run specs and print each outcome and time from the TRX
  bb mcp list <solution>    list MCP server projects and run commands
  bb mcp tools <project>    list tools from one MCP server
  bb mcp call <project> <tool> '<json>'
                            call one tool with JSON arguments
  bb mcp log <project>      follow timestamped server stderr";

/// Default measured runs per bench item.
pub const DEFAULT_RUNS: usize = 5;
/// Most measured runs per bench item.
pub const MAX_RUNS: usize = 100;

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
        /// Measured runs per item, after one warm-up.
        runs: usize,
    },
    /// MCP discovery and client commands.
    Mcp(McpCommand),
    /// Ask Roslyn one feature at one position.
    Probe(Probe),
    /// List or run tests.
    Test(TestArgs),
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
    /// `--runs` needs a whole number from 1 to [`MAX_RUNS`].
    #[error("--runs needs a number from 1 to {MAX_RUNS}, got {0}")]
    BadRuns(String),
    /// An argument the command does not take.
    #[error("unexpected argument: {0}")]
    Extra(String),
    /// Unknown MCP subcommand.
    #[error("unknown mcp command: {0}")]
    McpCommand(String),
    /// A position must be `line:col`, both from 1.
    #[error("position must be line:col, both from 1, got {0}")]
    BadPoint(String),
    /// An unknown probe feature.
    #[error("unknown feature {0}: use completion, hover, definition, references, rename=<NewName>, actions or diagnostics")]
    BadFeature(String),
    /// A `--specs` entry is not a test name.
    #[error("--specs: {0}")]
    BadSpec(String),
    /// `--repeat` needs a whole number from 1 to [`MAX_REPEAT`].
    #[error("--repeat needs a number from 1 to {MAX_REPEAT}, got {0}")]
    BadRepeat(String),
}

/// Parse the arguments after the program name.
///
/// # Errors
/// [`CliError`] for a missing or unknown command, a missing path, or an
/// argument the command does not take.
pub fn parse(args: &[String]) -> Result<Command, CliError> {
    let mut it: std::slice::Iter<'_, String> = args.iter();
    let word: &String = it.next().ok_or(CliError::Missing)?;
    if word == "bench" {
        return bench(it.as_slice());
    }
    if word == "mcp" {
        return mcp::parse(it.as_slice()).map(Command::Mcp);
    }
    if word == "probe" {
        return bb_probe::parse(it.as_slice()).map(Command::Probe);
    }
    if word == "test" {
        return test_args::parse(it.as_slice()).map(Command::Test);
    }
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
        other => Err(CliError::Unknown(other.to_owned())),
    }
}

/// `bench [solution] [--runs N]`, in that order.
fn bench(rest: &[String]) -> Result<Command, CliError> {
    let (solution, flags): (Option<PathBuf>, &[String]) = match rest.split_first() {
        Some((first, after)) if first != "--runs" => (Some(PathBuf::from(first)), after),
        Some(_) | None => (None, rest),
    };
    let runs: usize = match flags {
        [] => DEFAULT_RUNS,
        [flag, n] if flag == "--runs" => n
            .parse::<usize>()
            .ok()
            .filter(|r: &usize| (1..=MAX_RUNS).contains(r))
            .ok_or_else(|| CliError::BadRuns(n.clone()))?,
        [flag] if flag == "--runs" => return Err(CliError::BadRuns(String::new())),
        [.., last] => return Err(CliError::Extra(last.clone())),
    };
    Ok(Command::Bench { solution, runs })
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
mod tests;
