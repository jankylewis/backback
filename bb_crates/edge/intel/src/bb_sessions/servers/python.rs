//! A session on pyright (BBEOD28): found as `bb_python::pyright` says,
//! started in its own process group with the folder's interpreter as
//! `python.pythonPath`, writing no bytecode into the project.

use std::path::PathBuf;
use std::time::Duration;

use bb_lsp::Settings;
use bb_python::pyright::{self, Found, Inputs};
use serde_json::{json, Value};

use super::start::{start, Start};
use crate::session::Server;
use crate::{IntelError, Session};

/// How to start a pyright [`Session`].
#[derive(Clone, Debug)]
pub struct PythonConfig {
    /// The folder it serves (absolute).
    pub root: PathBuf,
    /// The interpreter pyright resolves imports with (never run by it).
    pub interpreter: PathBuf,
    /// What discovery reads ([`Inputs::from_env`] in the app).
    pub inputs: Inputs,
    /// Default deadline for one request.
    pub request_timeout: Duration,
}

impl Session {
    /// Find pyright, start it on `config.root` and initialize it.
    ///
    /// # Errors
    /// [`IntelError::Python`] when no pyright runs (its text is the
    /// install hint), a bad path, or `initialize` fails.
    pub async fn start_python<W: std::io::Write + Send + 'static>(
        config: &PythonConfig,
        log: W,
    ) -> Result<Self, IntelError> {
        let found: Found = pyright::locate(&config.inputs)?;
        let mut settings: Settings = Settings::new();
        for (section, value) in pyright::settings(&config.interpreter) {
            settings.insert(section, value);
        }
        let options: Value = json!({ "python": { "pythonPath": config.interpreter } });
        let begin: Start = Start {
            server: Server::Pyright,
            command: pyright::command(&found.server),
            settings,
            root: config.root.clone(),
            options,
            request_timeout: config.request_timeout,
        };
        start(begin, log).await
    }
}

#[cfg(test)]
#[path = "server_tests/python.rs"]
mod tests;
