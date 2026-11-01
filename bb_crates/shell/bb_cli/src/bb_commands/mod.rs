//! Command implementations. Each prints plain lines so the output can be
//! pasted into docs as evidence.

mod bb_open;
mod bench;
mod dotnet;
mod probe;

pub(crate) use dotnet::{mcp, scan};

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_dotnet::Sdk;
use bb_lsp::roslyn;
use bb_ui::{Theme, ThemeKind};

use crate::cli::Command;

/// Deadline for short `dotnet` calls.
pub(crate) const PROCESS_TIMEOUT: Duration = Duration::from_secs(30);

/// Command errors.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// .NET SDK problem.
    #[error(transparent)]
    Sdk(#[from] bb_dotnet::SdkError),
    /// Solution problem.
    #[error(transparent)]
    Solution(#[from] bb_dotnet::SolutionError),
    /// A project file could not be read.
    #[error(transparent)]
    Project(#[from] bb_dotnet::ProjectError),
    /// Some projects of a solution could not be read (each already printed).
    #[error("{failed} of {total} projects could not be read")]
    Projects {
        /// Projects that failed.
        failed: usize,
        /// Projects the solution declares.
        total: usize,
    },
    /// Roslyn problem.
    #[error(transparent)]
    Roslyn(#[from] roslyn::RoslynError),
    /// Syntax problem.
    #[error(transparent)]
    Syntax(#[from] bb_syntax::SyntaxError),
    /// Audit could not read the tree.
    #[error(transparent)]
    Audit(#[from] bb_audit::AuditError),
    /// `bb bench` problem.
    #[error(transparent)]
    Bench(#[from] bench::BenchError),
    /// MCP problem.
    #[error(transparent)]
    Mcp(#[from] mcp::McpCommandError),
    /// `bb probe` problem.
    #[error(transparent)]
    Probe(#[from] probe::ProbeError),
    /// `bb test` problem.
    #[error(transparent)]
    Test(#[from] dotnet::TestError),
    /// `bb doctor` found a required tool missing or broken (already printed).
    #[error("required tools not usable: {}", .0.join(", "))]
    Doctor(Vec<&'static str>),
    /// The audit found violations (already printed).
    #[error("{0} convention violations")]
    Violations(usize),
    /// File read problem.
    #[error("cannot read {path}: {source}")]
    Read {
        /// Path.
        path: PathBuf,
        /// Cause.
        #[source]
        source: std::io::Error,
    },
}

/// Run one command.
///
/// # Errors
/// Whatever the command reports; see [`CommandError`].
pub async fn run(command: Command) -> Result<(), CommandError> {
    match command {
        Command::Doctor => doctor().await,
        Command::Scan { solution } => scan::run(&solution),
        Command::Open { solution, restore } => bb_open::run(&solution, restore).await,
        Command::Outline { file } => outline(&file),
        Command::Audit { root } => audit(&root),
        Command::Bench { solution, runs } => bench::run(solution.as_deref(), runs),
        Command::Mcp(command) => mcp::run(command).await.map_err(CommandError::from),
        Command::Probe(probe) => probe::run(&probe).await,
        Command::Test(args) => dotnet::test(&args).await,
    }
}

async fn doctor() -> Result<(), CommandError> {
    let theme: Theme = Theme::for_kind(None);
    println!("theme     : {} ({:?})", theme.name, theme.kind);
    println!(
        "light ok  : {}",
        Theme::for_kind(Some(ThemeKind::Light)).name
    );
    let mut broken: Vec<&'static str> = Vec::new();
    match Sdk::locate() {
        Ok(sdk) => {
            println!("dotnet    : {}", sdk.host().display());
            match sdk.version(PROCESS_TIMEOUT).await {
                Ok(v) => println!("sdk       : {}", v.as_str()),
                Err(e) => {
                    println!("sdk       : ERROR {e}");
                    broken.push("dotnet sdk");
                }
            }
        }
        Err(e) => {
            println!("dotnet    : MISSING ({e})");
            broken.push("dotnet");
        }
    }
    match roslyn::locate() {
        Ok(p) => println!("roslyn-ls : {}", p.display()),
        Err(e) => {
            println!("roslyn-ls : MISSING ({e})");
            broken.push("roslyn-ls");
        }
    }
    match bb_dap::locate() {
        Ok(p) => println!("netcoredbg: {}", p.display()),
        Err(e) => println!("netcoredbg: OPTIONAL (needed from BBEOD05): {e}"),
    }
    if broken.is_empty() {
        Ok(())
    } else {
        Err(CommandError::Doctor(broken))
    }
}

fn outline(file: &Path) -> Result<(), CommandError> {
    let text: String = read(file)?;
    let mut syntax: bb_syntax::CSharpSyntax = bb_syntax::CSharpSyntax::new()?;
    let started: Instant = Instant::now();
    let tree: bb_syntax::Tree = syntax.parse(&text, None)?;
    let parsed_in: Duration = started.elapsed();
    for item in bb_syntax::CSharpSyntax::outline(&tree, &text) {
        let indent: String = "  ".repeat(item.depth);
        println!(
            "{indent}{:?} {} (line {})",
            item.kind,
            item.name,
            item.line.saturating_add(1)
        );
    }
    println!("parsed    : {} bytes in {parsed_in:?}", text.len());
    Ok(())
}

fn audit(root: &Path) -> Result<(), CommandError> {
    let found: Vec<bb_audit::Violation> = bb_audit::audit(root)?;
    for v in &found {
        println!("{v}");
    }
    if found.is_empty() {
        println!("audit     : clean ({})", root.display());
        Ok(())
    } else {
        Err(CommandError::Violations(found.len()))
    }
}

/// Read a file as text with a typed error.
pub(crate) fn read(file: &Path) -> Result<String, CommandError> {
    std::fs::read_to_string(file).map_err(|source: std::io::Error| CommandError::Read {
        path: file.to_path_buf(),
        source,
    })
}
