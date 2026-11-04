//! `bb open`: start Roslyn from Rust, load a solution, time it.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_dotnet::{Solution, SolutionPath};
use bb_lsp::roslyn;
use bb_lsp::{Incoming, LspClient};
use lsp_types::InitializeResult;
use tokio::sync::mpsc;

use super::super::CommandError;

const LSP_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const SOLUTION_LOAD_TIMEOUT: Duration = Duration::from_secs(300);
/// How long the server's last lines may take after `shutdown`.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
/// How long `--restore` waits for Roslyn's restore to end after the load.
const RESTORE_TIMEOUT: Duration = Duration::from_secs(120);

pub(crate) async fn run(solution: &Path, restore: roslyn::Restore) -> Result<(), CommandError> {
    let (sln, listing): (SolutionPath, Solution) = describe(solution, restore)?;
    let binary: PathBuf = roslyn::locate()?;
    let log_dir: PathBuf = std::env::temp_dir().join("backback-roslyn-logs");
    let owned: bb_process::OwnedTempDir = scratch(sln.as_path())?;
    let scratch: PathBuf = owned.path().to_path_buf();
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
    let (log_path, mut log): (PathBuf, std::fs::File) = super::logs::create_log(&log_dir)?;
    let load: roslyn::Loaded = roslyn::wait_for_load(
        &mut incoming,
        SOLUTION_LOAD_TIMEOUT,
        &listing.projects,
        &mut log,
    )
    .await?;
    report_load(&load, &listing, started, &log_path);
    finish_restore(
        restore,
        &scratch,
        (&mut incoming, &mut log),
        load.restore_ended,
    )
    .await?;
    let stopped: Result<(), bb_lsp::LspError> = client.shutdown().await;
    roslyn::drain_log(&mut incoming, DRAIN_TIMEOUT, &mut log).await?;
    drop(owned);
    stopped.map_err(roslyn::RoslynError::Client)?;
    Ok(())
}

/// The load's outcome lines: time, failed projects, total, log path.
fn report_load(load: &roslyn::Loaded, listing: &Solution, started: Instant, log_path: &Path) {
    let total: usize = roslyn::LoadState::new(&listing.projects).total();
    println!(
        "loaded    : {:?} until all {total} projects loaded",
        load.took
    );
    for failed in &load.failed {
        println!("failed    : {}", failed.as_path().display());
    }
    println!("total     : {:?}", started.elapsed());
    println!("log       : {}", log_path.display());
}

/// Restore off: name the scratch folder. Restore on: wait until Roslyn's
/// restore has ended, so the run log holds it (BB-LSP-004); a restore that
/// ended during the load is in the log already (BB-LSP-006).
async fn finish_restore(
    restore: roslyn::Restore,
    scratch: &Path,
    (incoming, log): (&mut mpsc::Receiver<Incoming>, &mut std::fs::File),
    ended_during_load: bool,
) -> Result<(), CommandError> {
    match restore {
        roslyn::Restore::Off => println!("obj       : {} (removed at exit)", scratch.display()),
        roslyn::Restore::On if ended_during_load => {
            println!("restored  : true (ended during the load)");
        }
        roslyn::Restore::On => {
            let started: Instant = Instant::now();
            let ended: bool = roslyn::wait_for_restore(incoming, RESTORE_TIMEOUT, log).await?;
            println!("restored  : {ended} after {:?}", started.elapsed());
        }
    }
    Ok(())
}

/// Read the solution and print what is being opened.
fn describe(
    solution: &Path,
    restore: roslyn::Restore,
) -> Result<(SolutionPath, Solution), CommandError> {
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
    Ok((sln, listing))
}

/// Where the design-time build writes when restore is off: one owned folder
/// per solution and per run, `backback-obj-<pid>-<solution>`, outside the
/// solution, so parallel runs never share build files. Dropping it removes
/// it on every exit path; a crashed run's folder is swept (BB-PRC-006).
pub(crate) fn scratch(solution: &Path) -> Result<bb_process::OwnedTempDir, CommandError> {
    let key: String = scratch_key(solution);
    bb_process::OwnedTempDir::create_with("backback-obj", &key).map_err(|source: std::io::Error| {
        CommandError::Read {
            path: std::env::temp_dir().join(format!("backback-obj-{key}")),
            source,
        }
    })
}

/// The solution path as one folder-name part: no `-`, so the sweep reads
/// the pid before it.
fn scratch_key(solution: &Path) -> String {
    solution
        .to_string_lossy()
        .chars()
        .map(|c: char| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_solution_gets_its_own_owned_scratch_folder() {
        let sln: &Path = Path::new("/r/A-1.slnx");
        assert_ne!(scratch_key(Path::new("/r/B.slnx")), scratch_key(sln));
        assert!(
            !scratch_key(sln).contains('-'),
            "the pid stays the first digit part"
        );
        let dir: bb_process::OwnedTempDir = scratch(sln).unwrap();
        let path: PathBuf = dir.path().to_path_buf();
        let name: String = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with(&format!("backback-obj-{}-", std::process::id())));
        assert!(path.is_dir());
        drop(dir);
        assert!(!path.exists(), "drop removes it");
    }
}
