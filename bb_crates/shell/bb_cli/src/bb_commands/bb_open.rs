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
    let scratch: PathBuf = scratch_dir(sln.as_path(), std::process::id());
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
        println!("obj       : {} (removed at exit)", scratch.display());
    }
    let stopped: Result<(), bb_lsp::LspError> = client.shutdown().await;
    // Best effort: a scratch folder left behind is only temp space.
    let _removed: std::io::Result<()> = std::fs::remove_dir_all(&scratch);
    stopped.map_err(roslyn::RoslynError::Client)?;
    Ok(())
}

/// Most names [`create_log`] tries before it gives up.
const LOG_NAME_TRIES: u32 = 1000;

/// Create this run's server log in `log_dir`: `open-<unix ms>-<pid>-<n>.log`,
/// created only if new, so two runs never write into one file.
fn create_log(log_dir: &Path) -> Result<(PathBuf, std::fs::File), CommandError> {
    let millis: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d: Duration| d.as_millis());
    let pid: u32 = std::process::id();
    let mut last: Option<(PathBuf, std::io::Error)> = None;
    for n in 0..LOG_NAME_TRIES {
        let path: PathBuf = log_dir.join(format!("open-{millis}-{pid}-{n}.log"));
        match std::fs::File::options()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => last = Some((path, e)),
            Err(source) => return Err(CommandError::Read { path, source }),
        }
    }
    let (path, source): (PathBuf, std::io::Error) = last.unwrap_or_else(|| {
        (
            log_dir.to_path_buf(),
            std::io::Error::from(std::io::ErrorKind::AlreadyExists),
        )
    });
    Err(CommandError::Read { path, source })
}

/// Where the design-time build writes when restore is off: one folder per
/// solution and per run (`pid`), outside the solution, so parallel runs never
/// share build files. The run removes it when it ends.
fn scratch_dir(solution: &Path, pid: u32) -> PathBuf {
    let key: String = solution
        .to_string_lossy()
        .chars()
        .map(|c: char| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    std::env::temp_dir()
        .join("backback-obj")
        .join(format!("{key}-{pid}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_logs_created_in_one_millisecond_never_share_a_file() {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("bb-open-logs-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (first, mut file): (PathBuf, std::fs::File) = create_log(&dir).unwrap();
        std::io::Write::write_all(&mut file, b"first run\n").unwrap();
        let (second, _other): (PathBuf, std::fs::File) = create_log(&dir).unwrap();
        let kept: String = std::fs::read_to_string(&first).unwrap();
        let _gone: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        assert_ne!(first, second);
        assert_eq!(kept, "first run\n");
    }

    #[test]
    fn parallel_runs_on_one_solution_get_their_own_scratch_folder() {
        let sln: &Path = Path::new("/r/A.slnx");
        assert_ne!(scratch_dir(sln, 1), scratch_dir(sln, 2));
        assert_ne!(scratch_dir(Path::new("/r/B.slnx"), 1), scratch_dir(sln, 1));
    }
}
