//! The session: one server, one loaded solution (or Cargo workspace), the
//! open documents.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bb_dotnet::{ProjectPath, Solution, SolutionPath};
use bb_lsp::roslyn::{self, Restore};
use bb_lsp::{Incoming, LspClient, LspError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::sync::Notify;

use super::{pump, Server};
use crate::model::bb_doc::{DocVersion, OpenDoc};
use crate::model::store::Store;
use crate::model::DocPath;
use crate::IntelError;

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
    pub(crate) scratch: Option<PathBuf>,
    pub(crate) load: roslyn::Loaded,
    pub(crate) pump: tokio::task::JoinHandle<()>,
    pub(crate) refresh: Arc<Notify>,
    pub(crate) server: Server,
}

/// How long the server's last log lines may take after `shutdown`.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
/// LSP `ContentModified`: the server dropped the request because its state
/// changed meanwhile (rust-analyzer while it reloads its workspace).
const CONTENT_MODIFIED: i64 = -32801;
/// The pause before asking again after [`CONTENT_MODIFIED`].
const SETTLE: Duration = Duration::from_millis(200);

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
        let load: roslyn::Loaded = roslyn::wait_for_load(
            &mut incoming,
            config.load_timeout,
            &listing.projects,
            &mut log,
        )
        .await?;
        let store: Store = Store::default();
        let refresh: Arc<Notify> = Arc::new(Notify::new());
        let pump: tokio::task::JoinHandle<()> = tokio::spawn(pump(
            incoming,
            (store.clone(), Arc::clone(&refresh)),
            Server::Roslyn,
            log,
        ));
        Ok(Self {
            client,
            docs: HashMap::new(),
            store,
            request_timeout: config.request_timeout,
            scratch: Some(config.scratch.clone()),
            load,
            pump,
            refresh,
            server: Server::Roslyn,
        })
    }

    /// How long the solution took to load.
    #[must_use]
    pub fn loaded_in(&self) -> Duration {
        self.load.took
    }

    /// Projects Roslyn reported it could not load (their reasons are in
    /// the server log).
    #[must_use]
    pub fn failed_projects(&self) -> &[ProjectPath] {
        &self.load.failed
    }

    /// Signal when the server closes its connection.
    #[must_use]
    pub fn closed_signal(&self) -> watch::Receiver<bool> {
        self.client.closed_signal()
    }

    /// Fires when every pulled diagnostics list may be stale (BB-BUG-019):
    /// the server asked for a refresh, or it (re)loaded projects. A permit is
    /// kept while nobody waits, so a refresh is never lost.
    #[must_use]
    pub fn refresh_signal(&self) -> Arc<Notify> {
        Arc::clone(&self.refresh)
    }

    /// Which server this session runs.
    #[must_use]
    pub fn server(&self) -> Server {
        self.server
    }

    /// Every open document.
    #[must_use]
    pub fn open_paths(&self) -> Vec<PathBuf> {
        self.docs
            .keys()
            .map(|doc: &DocPath| doc.as_path().to_path_buf())
            .collect()
    }

    /// Kill the server child as a crash would, for an instrument test.
    ///
    /// # Errors
    /// The child cannot be killed within the request deadline.
    pub async fn kill_child(&mut self) -> Result<(), IntelError> {
        Ok(self.client.kill_child().await?)
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

    /// A raw request with its own deadline; every feature goes through it.
    /// A [`CONTENT_MODIFIED`] reply is asked again while the deadline lasts
    /// (BBEOD29), so a request made while the server settles still answers.
    ///
    /// # Errors
    /// See [`LspClient::request_within`]; past the deadline, the server's
    /// last [`CONTENT_MODIFIED`] error.
    pub async fn raw<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
        deadline: Duration,
    ) -> Result<R, IntelError> {
        let params: Value = serde_json::to_value(params).map_err(LspError::Json)?;
        let asked: Instant = Instant::now();
        loop {
            let left: Duration = deadline.saturating_sub(asked.elapsed());
            match self
                .client
                .request_within(method, params.clone(), left)
                .await
            {
                Err(LspError::Server { code, .. })
                    if code == CONTENT_MODIFIED && asked.elapsed() + SETTLE < deadline =>
                {
                    tokio::time::sleep(SETTLE).await;
                }
                answered => return Ok(answered?),
            }
        }
    }

    /// Stop the server, wait (bounded) until its last log lines are
    /// written, and remove the scratch folder (Roslyn's; rust-analyzer's
    /// target folder is kept, so the next start does not rebuild it).
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
        if let Some(scratch) = &self.scratch {
            let _removed: std::io::Result<()> = std::fs::remove_dir_all(scratch);
        }
        Ok(stopped?)
    }

    pub(crate) fn doc(&self, path: &DocPath) -> Result<&OpenDoc, IntelError> {
        self.docs
            .get(path)
            .ok_or_else(|| IntelError::NotOpen(path.as_path().to_path_buf()))
    }
}

#[cfg(test)]
#[path = "servers/server_tests/settle.rs"]
mod settle_tests;
