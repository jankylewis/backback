//! How a server next to Roslyn starts (rust-analyzer, pyright): spawned
//! in its own process group, initialized on one folder, its messages
//! pumped into the diagnostics store. One path for every such server.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bb_lsp::{roslyn, Incoming, LspClient, Settings};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio::sync::Notify;
use url::Url;

use crate::model::store::Store;
use crate::session::{pump, Server};
use crate::{IntelError, Session};

/// What a server needs to start.
pub(crate) struct Start {
    /// Which server.
    pub(crate) server: Server,
    /// Its command (stdio), with its environment.
    pub(crate) command: std::process::Command,
    /// Its `workspace/configuration` answers.
    pub(crate) settings: Settings,
    /// The folder it serves (absolute).
    pub(crate) root: PathBuf,
    /// Its `initializationOptions`.
    pub(crate) options: Value,
    /// Default deadline for one request.
    pub(crate) request_timeout: Duration,
}

/// Spawn, initialize and pump `start`'s server; server log lines go to `log`.
pub(crate) async fn start<W: std::io::Write + Send + 'static>(
    start: Start,
    log: W,
) -> Result<Session, IntelError> {
    let started: Instant = Instant::now();
    let command: tokio::process::Command = tokio::process::Command::from(start.command);
    let mut client: LspClient = LspClient::spawn(command, start.request_timeout, start.settings)?;
    let incoming: mpsc::Receiver<Incoming> = client
        .take_incoming()
        .ok_or(roslyn::RoslynError::NoIncoming)?;
    initialize(&client, &start.root, start.options).await?;
    let store: Store = Store::default();
    let refresh: Arc<Notify> = Arc::new(Notify::new());
    let pump: tokio::task::JoinHandle<()> = tokio::spawn(pump(
        incoming,
        (store.clone(), Arc::clone(&refresh)),
        start.server,
        log,
    ));
    Ok(Session {
        client,
        docs: HashMap::new(),
        store,
        request_timeout: start.request_timeout,
        scratch: None,
        load: roslyn::Loaded {
            took: started.elapsed(),
            failed: Vec::new(),
            restore_ended: false,
        },
        pump,
        refresh,
        server: start.server,
    })
}

/// `initialize` (root, workspace folder, `options` as
/// `initializationOptions`) then `initialized`.
async fn initialize(client: &LspClient, root: &Path, options: Value) -> Result<(), IntelError> {
    let uri: Url =
        Url::from_directory_path(root).map_err(|(): ()| IntelError::BadPath(root.to_path_buf()))?;
    let name: String = root
        .file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let params: Value = json!({
        "processId": std::process::id(),
        "rootUri": uri,
        "workspaceFolders": [{ "uri": uri, "name": name }],
        "initializationOptions": options,
        "capabilities": {
            "workspace": { "configuration": true },
            "textDocument": {
                "publishDiagnostics": { "versionSupport": true },
                "hover": { "contentFormat": ["markdown", "plaintext"] }
            }
        }
    });
    let _result: Value = client.request("initialize", params).await?;
    client.notify("initialized", json!({})).await?;
    Ok(())
}
