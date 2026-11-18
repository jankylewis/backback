//! The load phase of the server thread: start the session, report
//! progress, then `Ready` with the failed projects, or `Failed`.

use std::path::PathBuf;
use std::time::Duration;

use bb_intel::{Config, IntelError, RustConfig, Session};
use bb_lsp::roslyn::LoadState;
use tokio::sync::mpsc::UnboundedSender;

use super::{send, Boot};
use crate::intel::bb_worker::progress::ProgressLog;
use crate::intel::bb_worker::protocol::Event;

/// Start the session and report the load. `None` when it failed.
pub(super) async fn load(boot: Boot, events: &UnboundedSender<Event>) -> Option<Session> {
    match boot {
        Boot::Roslyn(config, projects) => load_roslyn(&config, projects, events).await,
        Boot::Rust(config, log_dir) => load_rust(&config, &log_dir, events).await,
    }
}

/// Start rust-analyzer: `Ready` once initialized, `Unavailable` with the
/// install hint when it is not installed, else `Failed`.
async fn load_rust(
    config: &RustConfig,
    log_dir: &std::path::Path,
    events: &UnboundedSender<Event>,
) -> Option<Session> {
    let log: std::fs::File = match std::fs::File::create(log_dir.join("backback-rust-analyzer.log"))
    {
        Ok(file) => file,
        Err(error) => {
            send(events, Event::Failed(format!("run log: {error}")));
            return None;
        }
    };
    match Session::start_rust(config, log).await {
        Ok(session) => {
            let took: Duration = session.loaded_in();
            send(
                events,
                Event::Ready {
                    took,
                    failed: Vec::new(),
                },
            );
            Some(session)
        }
        Err(IntelError::Cargo(missing @ bb_cargo::CargoError::AnalyzerMissing)) => {
            send(events, Event::Unavailable(missing.to_string()));
            None
        }
        Err(error) => {
            send(events, Event::Failed(error.to_string()));
            None
        }
    }
}

/// Start Roslyn and report the load: progress, then `Ready` with the
/// projects that failed, or `Failed`. `None` when the load failed.
async fn load_roslyn(
    config: &Config,
    projects: LoadState,
    events: &UnboundedSender<Event>,
) -> Option<Session> {
    let log: std::fs::File = match std::fs::File::create(config.log_dir.join("backback-roslyn.log"))
    {
        Ok(file) => file,
        Err(error) => {
            send(events, Event::Failed(format!("run log: {error}")));
            return None;
        }
    };
    let total: usize = projects.total();
    send(events, Event::Progress { loaded: 0, total });
    let started: Result<Session, IntelError> =
        Session::start(config, ProgressLog::new(log, events.clone(), projects)).await;
    let session: Session = match started {
        Ok(session) => session,
        Err(error) => {
            send(events, Event::Failed(error.to_string()));
            return None;
        }
    };
    let failed: Vec<PathBuf> = session
        .failed_projects()
        .iter()
        .map(|p: &bb_dotnet::ProjectPath| p.as_path().to_path_buf())
        .collect();
    let took: Duration = session.loaded_in();
    send(events, Event::Ready { took, failed });
    Some(session)
}
