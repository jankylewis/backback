//! The load phase of the server thread: start the session, report
//! progress, then `Ready` with the failed projects, or `Failed`.

use std::path::PathBuf;
use std::time::Duration;

use bb_intel::{Config, IntelError, Session};
use bb_lsp::roslyn::LoadState;
use tokio::sync::mpsc::UnboundedSender;

use super::{send, Boot};
use crate::intel::bb_worker::progress::ProgressLog;
use crate::intel::bb_worker::protocol::Event;

/// Start the session and report the load. `None` when it failed.
pub(super) async fn load(boot: Boot, events: &UnboundedSender<Event>) -> Option<Session> {
    match boot {
        Boot::Roslyn(config, projects) => load_roslyn(&config, projects, events).await,
        Boot::Rust(config, log_dir) => {
            let log: std::fs::File = side_log(&log_dir, "backback-rust-analyzer.log", events)?;
            side_started(Session::start_rust(&config, log).await, events)
        }
        Boot::Python(config, log_dir) => {
            let log: std::fs::File = side_log(&log_dir, "backback-pyright.log", events)?;
            side_started(Session::start_python(&config, log).await, events)
        }
    }
}

/// The run log of a side server, or `Failed` with why.
fn side_log(
    log_dir: &std::path::Path,
    name: &str,
    events: &UnboundedSender<Event>,
) -> Option<std::fs::File> {
    match std::fs::File::create(log_dir.join(name)) {
        Ok(file) => Some(file),
        Err(error) => {
            send(events, Event::Failed(format!("run log: {error}")));
            None
        }
    }
}

/// Report a side server's start (rust-analyzer, pyright): `Ready` once
/// initialized, `Unavailable` with the install hint when it is not
/// installed, else `Failed`.
fn side_started(
    started: Result<Session, IntelError>,
    events: &UnboundedSender<Event>,
) -> Option<Session> {
    match started {
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
        Err(
            missing @ (IntelError::Cargo(bb_cargo::CargoError::AnalyzerMissing)
            | IntelError::Python(bb_python::PythonError::PyrightMissing)),
        ) => {
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
