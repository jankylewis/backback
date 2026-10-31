//! `bb open`: start Roslyn from Rust, load a solution, time it.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_dotnet::{Solution, SolutionPath};
use bb_lsp::roslyn;
use bb_lsp::{Incoming, LspClient};
use lsp_types::InitializeResult;
use tokio::sync::mpsc;

use super::CommandError;

const LSP_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const SOLUTION_LOAD_TIMEOUT: Duration = Duration::from_secs(300);

pub(crate) async fn run(solution: &Path) -> Result<(), CommandError> {
    let absolute: PathBuf =
        std::path::absolute(solution).map_err(|source: std::io::Error| CommandError::Read {
            path: solution.to_path_buf(),
            source,
        })?;
    let sln: SolutionPath = SolutionPath::new(absolute)?;
    let listing: Solution = Solution::load(&sln)?;
    println!(
        "solution  : {} ({:?})",
        sln.as_path().display(),
        sln.format()
    );
    println!("projects  : {}", listing.projects.len());
    let binary: PathBuf = roslyn::locate()?;
    let log_dir: PathBuf = std::env::temp_dir().join("backback-roslyn-logs");
    let root: &Path = sln.as_path().parent().unwrap_or_else(|| Path::new("/"));
    let started: Instant = Instant::now();
    let mut client: LspClient = roslyn::start(&binary, &log_dir, LSP_REQUEST_TIMEOUT)?;
    let mut incoming: mpsc::Receiver<Incoming> = client
        .take_incoming()
        .ok_or(roslyn::RoslynError::NoIncoming)?;
    let init: InitializeResult = roslyn::initialize(&client, root).await?;
    let server: String = init
        .server_info
        .map(|s: lsp_types::ServerInfo| s.name)
        .unwrap_or_default();
    println!("initialize: {:?} (server {server})", started.elapsed());
    roslyn::open_solution(&client, sln.as_path()).await?;
    let loaded_in: Duration = roslyn::wait_for_load(&mut incoming, SOLUTION_LOAD_TIMEOUT).await?;
    println!(
        "loaded    : {loaded_in:?} until {}",
        roslyn::PROJECT_INIT_COMPLETE
    );
    println!("total     : {:?}", started.elapsed());
    println!("logs      : {}", log_dir.display());
    client
        .shutdown()
        .await
        .map_err(roslyn::RoslynError::Client)?;
    Ok(())
}
