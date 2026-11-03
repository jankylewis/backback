//! The language server runs on its own thread. The UI holds only the two
//! channel ends, so no UI handler can block on Roslyn.

mod bb_loop;
mod progress;
pub(crate) mod protocol;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_dotnet::{Solution, SolutionPath};
use bb_intel::Config;
use bb_lsp::roslyn::{LoadState, Restore};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use protocol::{Event, Request};

/// The UI's end of the server thread.
pub(crate) struct Link {
    requests: UnboundedSender<Request>,
}

impl Link {
    /// Wrap a request sender (the real thread's, or a test's).
    pub(crate) fn new(requests: UnboundedSender<Request>) -> Self {
        Self { requests }
    }

    /// Queue a request. Never blocks; `false` when the thread has stopped.
    pub(crate) fn send(&self, request: Request) -> bool {
        self.requests.send(request).is_ok()
    }
}

/// Start the server thread for `solution` with restore off. The events
/// begin with progress and end the load with `Ready` or `Failed`.
pub(crate) fn start(solution: &Path) -> (Link, UnboundedReceiver<Event>) {
    let (request_tx, request_rx): (UnboundedSender<Request>, UnboundedReceiver<Request>) =
        unbounded_channel();
    let (event_tx, event_rx): (UnboundedSender<Event>, UnboundedReceiver<Event>) =
        unbounded_channel();
    let solution: PathBuf = solution.to_path_buf();
    let spawned: std::io::Result<std::thread::JoinHandle<()>> = std::thread::Builder::new()
        .name(String::from("bb-intel"))
        .spawn(move || serve(&solution, request_rx, &event_tx));
    if let Err(error) = spawned {
        let (failed_tx, failed_rx): (UnboundedSender<Event>, UnboundedReceiver<Event>) =
            unbounded_channel();
        // The receiver is still in this scope, so the send cannot fail.
        let _sent: Result<(), tokio::sync::mpsc::error::SendError<Event>> =
            failed_tx.send(Event::Failed(format!("server thread: {error}")));
        return (Link::new(request_tx), failed_rx);
    }
    (Link::new(request_tx), event_rx)
}

fn serve(solution: &Path, requests: UnboundedReceiver<Request>, events: &UnboundedSender<Event>) {
    match prepare(solution) {
        // `_folder` lives as long as the loop: its drop removes the logs.
        Ok((_folder, config, projects)) => bb_loop::run(&config, projects, requests, events),
        Err(reason) => {
            // Nobody listening means the window closed during the load.
            let _sent: Result<(), tokio::sync::mpsc::error::SendError<Event>> =
                events.send(Event::Failed(reason));
        }
    }
}

/// Track the solution's projects and make the log and scratch folders.
/// The folder is owned (BB-PRC-004): removed when the loop ends or the app
/// quits, and swept on the next start after a crash.
fn prepare(solution: &Path) -> Result<(bb_process::OwnedTempDir, Config, LoadState), String> {
    let listing: Solution = SolutionPath::new(solution.to_path_buf())
        .and_then(|path: SolutionPath| Solution::load(&path))
        .map_err(|error: bb_dotnet::SolutionError| error.to_string())?;
    let folder: bb_process::OwnedTempDir =
        intel_folder().map_err(|error: std::io::Error| format!("temp folder: {error}"))?;
    let base: &Path = folder.path();
    let (log_dir, scratch): (PathBuf, PathBuf) = (base.join("log"), base.join("scratch"));
    for dir in [&log_dir, &scratch] {
        std::fs::create_dir_all(dir)
            .map_err(|error: std::io::Error| format!("{}: {error}", dir.display()))?;
    }
    let config: Config = Config {
        solution: solution.to_path_buf(),
        restore: Restore::Off,
        log_dir,
        scratch,
        request_timeout: Duration::from_secs(20),
        load_timeout: Duration::from_secs(300),
    };
    Ok((folder, config, LoadState::new(&listing.projects)))
}

/// This worker's own folder, `backback-intel-<pid>-<n>`: one per worker, so
/// a second window (or a parallel test) never removes another's scratch
/// build and log (BB-BUG-038).
fn intel_folder() -> std::io::Result<bb_process::OwnedTempDir> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n: u64 = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    bb_process::OwnedTempDir::create_with("backback-intel", &n.to_string())
}

/// The solution to load for a workspace root: the argument itself when it
/// is one, else the first `.slnx` (then `.sln`) directly in the root.
pub(crate) fn find_solution(root: &Path) -> Option<PathBuf> {
    if is_solution(root) {
        return Some(root.to_path_buf());
    }
    let mut found: Vec<PathBuf> = std::fs::read_dir(root)
        .ok()?
        .filter_map(|entry: std::io::Result<std::fs::DirEntry>| {
            entry.ok().map(|e: std::fs::DirEntry| e.path())
        })
        .filter(|path: &PathBuf| is_solution(path))
        .collect();
    found.sort_by_key(|path: &PathBuf| {
        (
            path.extension() != Some(std::ffi::OsStr::new("slnx")),
            path.clone(),
        )
    });
    found.into_iter().next()
}

fn is_solution(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .is_some_and(|ext: &std::ffi::OsStr| ext == "slnx" || ext == "sln")
}

#[cfg(test)]
mod folder_tests {
    /// BB-BUG-038: two workers in one process get two folders; dropping
    /// one leaves the other whole.
    #[test]
    fn each_worker_owns_its_own_folder() {
        let first: bb_process::OwnedTempDir = super::intel_folder().unwrap();
        let second: bb_process::OwnedTempDir = super::intel_folder().unwrap();
        assert_ne!(first.path(), second.path());
        std::fs::write(first.path().join("backback-roslyn.log"), "x").unwrap();
        drop(second);
        assert!(
            first.path().join("backback-roslyn.log").is_file(),
            "the first worker's log stays"
        );
    }
}
