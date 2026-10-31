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

pub(crate) async fn run(solution: &Path, restore: roslyn::Restore) -> Result<(), CommandError> {
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
    println!("restore   : {restore:?}");
    let binary: PathBuf = roslyn::locate()?;
    let log_dir: PathBuf = std::env::temp_dir().join("backback-roslyn-logs");
    let scratch: PathBuf = scratch_dir(sln.as_path());
    let root: &Path = sln.as_path().parent().unwrap_or_else(|| Path::new("/"));
    let started: Instant = Instant::now();
    let mut client: LspClient =
        roslyn::start(&binary, &log_dir, LSP_REQUEST_TIMEOUT, restore, &scratch)?;
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
    let (log_path, mut log): (PathBuf, std::fs::File) = create_log(&log_dir)?;
    let loaded_in: Duration =
        roslyn::wait_for_load(&mut incoming, SOLUTION_LOAD_TIMEOUT, &mut log).await?;
    println!(
        "loaded    : {loaded_in:?} until {}",
        roslyn::PROJECT_INIT_COMPLETE
    );
    println!("total     : {:?}", started.elapsed());
    println!("log       : {}", log_path.display());
    if restore == roslyn::Restore::Off {
        println!("obj       : {}", scratch.display());
    }
    client
        .shutdown()
        .await
        .map_err(roslyn::RoslynError::Client)?;
    Ok(())
}

/// Create this run's server log in `log_dir`.
fn create_log(log_dir: &Path) -> Result<(PathBuf, std::fs::File), CommandError> {
    let path: PathBuf = log_dir.join(log_name());
    let file: std::fs::File =
        std::fs::File::create(&path).map_err(|source: std::io::Error| CommandError::Read {
            path: path.clone(),
            source,
        })?;
    Ok((path, file))
}

/// `open-<unix ms>.log`, so every run keeps its own server log.
fn log_name() -> String {
    let millis: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d: Duration| d.as_millis());
    format!("open-{millis}.log")
}

/// Where the design-time build writes for this solution when restore is off:
/// one folder per solution path, outside the solution.
fn scratch_dir(solution: &Path) -> PathBuf {
    let key: String = solution
        .to_string_lossy()
        .chars()
        .map(|c: char| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    std::env::temp_dir().join("backback-obj").join(key)
}
