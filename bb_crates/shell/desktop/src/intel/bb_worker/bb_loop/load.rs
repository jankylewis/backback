//! The load phase of the server thread: start the session, report
//! progress, then `Ready` with the failed projects, or `Failed`.

use std::path::PathBuf;
use std::time::Duration;

use bb_intel::{Config, IntelError, Session};
use bb_lsp::roslyn::LoadState;
use tokio::sync::mpsc::UnboundedSender;

use super::send;
use crate::intel::bb_worker::progress::ProgressLog;
use crate::intel::bb_worker::protocol::Event;

/// Start the session and report the load: progress, then `Ready` with the
/// projects that failed, or `Failed`. `None` when the load failed.
pub(super) async fn load(
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
