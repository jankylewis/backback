//! A session on rust-analyzer (BBEOD27): found as `bb_cargo::analyzer`
//! says, started in its own process group, with its `cargo check` and
//! build scripts building into backback's scratch folder, never the
//! project's `target/`.

use std::path::PathBuf;
use std::time::Duration;

use bb_cargo::analyzer::{self, Found, Inputs};
use bb_lsp::Settings;
use serde_json::Value;

use super::start::{start, Start};
use crate::session::Server;
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
        let found: Found = analyzer::locate(&config.inputs)?;
        std::fs::create_dir_all(&config.target_dir)
            .map_err(|_: std::io::Error| IntelError::BadPath(config.target_dir.clone()))?;
        let options: Value = analyzer::settings(&config.target_dir);
        let mut settings: Settings = Settings::new();
        settings.insert(String::from("rust-analyzer"), options.clone());
        let begin: Start = Start {
            server: Server::RustAnalyzer,
            command: analyzer::command(&found.binary, &config.target_dir),
            settings,
            root: config.root.clone(),
            options,
            request_timeout: config.request_timeout,
        };
        start(begin, log).await
    }
}

#[cfg(test)]
#[path = "server_tests/bb_rust.rs"]
mod tests;
