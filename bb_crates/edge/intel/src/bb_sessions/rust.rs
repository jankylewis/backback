//! A session on rust-analyzer (BBEOD27): found as `bb_cargo::analyzer`
//! says, started in its own process group, with its `cargo check` and
//! build scripts building into backback's scratch folder, never the
//! project's `target/`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bb_cargo::analyzer::{self, Found, Inputs};
use bb_lsp::{roslyn, Incoming, LspClient, Settings};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio::sync::Notify;
use url::Url;

use super::{pump, Server};
use crate::model::store::Store;
use crate::{IntelError, Session};

/// How to start a rust-analyzer [`Session`].
#[derive(Clone, Debug)]
pub struct RustConfig {
    /// The Cargo workspace folder (absolute).
    pub root: PathBuf,
    /// Where its `cargo check` builds: backback's scratch, never `root/target`.
    pub target_dir: PathBuf,
    /// What discovery reads ([`Inputs::from_env`] in the app).
    pub inputs: Inputs,
    /// Default deadline for one request.
    pub request_timeout: Duration,
}

impl Session {
    /// Find rust-analyzer, start it on `config.root` and initialize it.
    /// It indexes in the background; requests answer once it can.
    ///
    /// # Errors
    /// [`IntelError::Cargo`] when no rust-analyzer runs (its text is the
    /// install hint), a bad path, or `initialize` fails.
    pub async fn start_rust<W: std::io::Write + Send + 'static>(
        config: &RustConfig,
        log: W,
    ) -> Result<Self, IntelError> {
        let started: Instant = Instant::now();
        let found: Found = analyzer::locate(&config.inputs)?;
        std::fs::create_dir_all(&config.target_dir)
            .map_err(|_: std::io::Error| IntelError::BadPath(config.target_dir.clone()))?;
        let options: Value = analyzer::settings(&config.target_dir);
        let mut settings: Settings = Settings::new();
        settings.insert(String::from("rust-analyzer"), options.clone());
        let command: tokio::process::Command =
            tokio::process::Command::from(analyzer::command(&found.binary, &config.target_dir));
        let mut client: LspClient = LspClient::spawn(command, config.request_timeout, settings)?;
        let incoming: mpsc::Receiver<Incoming> = client
            .take_incoming()
            .ok_or(roslyn::RoslynError::NoIncoming)?;
        initialize(&client, &config.root, options).await?;
        let store: Store = Store::default();
        let refresh: Arc<Notify> = Arc::new(Notify::new());
        let pump: tokio::task::JoinHandle<()> = tokio::spawn(pump(
            incoming,
            (store.clone(), Arc::clone(&refresh)),
            Server::RustAnalyzer,
            log,
        ));
        Ok(Self {
            client,
            docs: HashMap::new(),
            store,
            request_timeout: config.request_timeout,
            scratch: None,
            load: roslyn::Loaded {
                took: started.elapsed(),
                failed: Vec::new(),
                restore_ended: false,
            },
            pump,
            refresh,
            server: Server::RustAnalyzer,
        })
    }
}

/// `initialize` (root, workspace folder, the target folder as
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

#[cfg(test)]
#[path = "rust_tests.rs"]
mod tests;
