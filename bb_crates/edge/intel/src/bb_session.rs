//! The session: one server, one loaded solution, the open documents.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bb_dotnet::{Solution, SolutionPath};
use bb_lsp::roslyn::{self, Restore};
use bb_lsp::{Incoming, LspClient};
use lsp_types::PublishDiagnosticsParams;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio::sync::watch;

use crate::model::bb_doc::{DocVersion, OpenDoc};
use crate::model::{DocPath, DocUri};
use crate::IntelError;

/// Diagnostics per document, as the server last reported them.
pub(crate) type Store = Arc<Mutex<HashMap<DocPath, Vec<lsp_types::Diagnostic>>>>;

/// How to start a [`Session`].
#[derive(Clone, Debug)]
pub struct Config {
    /// The `.sln` or `.slnx` to load (absolute).
    pub solution: PathBuf,
    /// Whether Roslyn may restore and build inside the solution.
    pub restore: Restore,
    /// Where the server writes its own logs.
    pub log_dir: PathBuf,
    /// Where the design-time build writes when restore is off.
    pub scratch: PathBuf,
    /// Default deadline for one request.
    pub request_timeout: Duration,
    /// Deadline for the whole solution load.
    pub load_timeout: Duration,
}

/// A running server with one solution loaded.
pub struct Session {
    pub(crate) client: LspClient,
    pub(crate) docs: HashMap<DocPath, OpenDoc>,
    pub(crate) store: Store,
    pub(crate) request_timeout: Duration,
    scratch: PathBuf,
    loaded_in: Duration,
    pump: tokio::task::JoinHandle<()>,
}

/// How long the server's last log lines may take after `shutdown`.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

impl Session {
    /// Start the server, load `config.solution` and wait until every project
    /// of it has loaded. Server log lines go to `log`, during the load and
    /// for as long as the session runs.
    ///
    /// # Errors
    /// The server is missing or fails to start, a path is bad, or the load
    /// passes its deadline.
    pub async fn start<W: std::io::Write + Send + 'static>(
        config: &Config,
        mut log: W,
    ) -> Result<Self, IntelError> {
        let solution: DocPath = DocPath::new(config.solution.clone())?;
        let listing: Solution = Solution::load(&SolutionPath::new(config.solution.clone())?)?;
        let root: &Path = solution
            .as_path()
            .parent()
            .unwrap_or_else(|| Path::new("/"));
        let binary: PathBuf = roslyn::locate()?;
        let mut client: LspClient = roslyn::start(
            &binary,
            &config.log_dir,
            config.request_timeout,
            config.restore,
            &config.scratch,
        )?;
        let mut incoming: mpsc::Receiver<Incoming> = client
            .take_incoming()
            .ok_or(roslyn::RoslynError::NoIncoming)?;
        roslyn::initialize(&client, root).await?;
        roslyn::open_solution(&client, solution.as_path()).await?;
        let loaded_in: Duration = roslyn::wait_for_load(
            &mut incoming,
            config.load_timeout,
            listing.projects.len(),
            &mut log,
        )
        .await?;
        let store: Store = Store::default();
        let pump: tokio::task::JoinHandle<()> =
            tokio::spawn(pump(incoming, Arc::clone(&store), log));
        Ok(Self {
            client,
            docs: HashMap::new(),
            store,
            request_timeout: config.request_timeout,
            scratch: config.scratch.clone(),
            loaded_in,
            pump,
        })
    }

    /// How long the solution took to load.
    #[must_use]
    pub fn loaded_in(&self) -> Duration {
        self.loaded_in
    }

    /// Signal when the server closes its connection.
    #[must_use]
    pub fn closed_signal(&self) -> watch::Receiver<bool> {
        self.client.closed_signal()
    }

    /// Stop the server child for an instrument test.
    ///
    /// # Errors
    /// The child cannot be stopped within the request deadline.
    pub async fn stop_child(&mut self) -> Result<(), IntelError> {
        Ok(self.client.stop_child().await?)
    }

    /// The mirror text of an open document.
    #[must_use]
    pub fn text(&self, path: &DocPath) -> Option<String> {
        self.docs.get(path).map(|d: &OpenDoc| d.buffer.to_text())
    }

    /// The version of an open document.
    #[must_use]
    pub fn version(&self, path: &DocPath) -> Option<DocVersion> {
        self.docs.get(path).map(|d: &OpenDoc| d.version)
    }

    /// A raw request with its own deadline, for what the typed API lacks.
    ///
    /// # Errors
    /// See [`LspClient::request_within`].
    pub async fn raw<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
        deadline: Duration,
    ) -> Result<R, IntelError> {
        Ok(self.client.request_within(method, params, deadline).await?)
    }

    /// Stop the server, wait (bounded) until its last log lines are
    /// written, and remove the scratch folder.
    ///
    /// # Errors
    /// The `exit` notification cannot be sent.
    pub async fn shutdown(self) -> Result<(), IntelError> {
        let stopped: Result<(), bb_lsp::LspError> = self.client.shutdown().await;
        // The pump ends when the server closes its stream; past the deadline
        // the log simply ends there, which is not a shutdown failure.
        let _drained: Result<Result<(), tokio::task::JoinError>, tokio::time::error::Elapsed> =
            tokio::time::timeout(DRAIN_TIMEOUT, self.pump).await;
        // Best effort: a scratch folder left behind is only temp space.
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(&self.scratch);
        Ok(stopped?)
    }

    pub(crate) fn doc(&self, path: &DocPath) -> Result<&OpenDoc, IntelError> {
        self.docs
            .get(path)
            .ok_or_else(|| IntelError::NotOpen(path.as_path().to_path_buf()))
    }
}

/// Keep reading server messages so the reader never blocks on a full
/// channel; write each one to the run log, and keep each pushed
/// `publishDiagnostics` per document.
async fn pump<W: std::io::Write>(mut incoming: mpsc::Receiver<Incoming>, store: Store, mut log: W) {
    while let Some(message) = incoming.recv().await {
        // A log that stops accepting lines must not stop the drain: the
        // reader would block and every later request would time out.
        let _logged: Result<&str, roslyn::RoslynError> = roslyn::log_incoming(&message, &mut log);
        let Incoming::Notification { method, params }: Incoming = message else {
            continue;
        };
        if method != "textDocument/publishDiagnostics" {
            continue;
        }
        let Some(report): Option<PublishDiagnosticsParams> =
            params.and_then(|p: serde_json::Value| serde_json::from_value(p).ok())
        else {
            continue;
        };
        if let DocUri::File(path) = DocUri::from_url(&report.uri) {
            if let Ok(mut map) = store.lock() {
                map.insert(path, report.diagnostics);
            }
        }
    }
}
