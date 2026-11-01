//! The five `apexTestkit.*` settings from a workspace `.vscode/settings.json`.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::jsonc::strip_jsonc;
use crate::model::Configuration;

/// The settings file below a workspace folder.
pub const SETTINGS_FILE: &str = ".vscode/settings.json";

/// Why the settings cannot be used.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// The file exists but cannot be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// The file.
        path: PathBuf,
        /// The cause.
        #[source]
        source: std::io::Error,
    },
    /// Not JSON, even after comments and trailing commas are removed.
    #[error("{path} is not JSON: {detail}")]
    Json {
        /// The file.
        path: PathBuf,
        /// The parser message.
        detail: String,
    },
    /// A key has the wrong type or an unusable value.
    #[error("{key} must be {wants}")]
    BadValue {
        /// The key.
        key: &'static str,
        /// What it must be.
        wants: &'static str,
    },
}

/// The command prefix, split on white space and run without a shell (as
/// the Testkit does: quotes are not shell quotes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLine {
    /// The program.
    pub program: String,
    /// Arguments before `test ...`.
    pub prefix: Vec<String>,
}

impl CommandLine {
    /// Split `text` on white space; `None` when it holds no word.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut words: std::str::SplitWhitespace<'_> = text.split_whitespace();
        let program: String = words.next()?.to_owned();
        Some(Self {
            program,
            prefix: words.map(str::to_owned).collect(),
        })
    }
}

/// `apexTestkit.*`, with the extension's defaults for absent keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApexSettings {
    /// `apexTestkit.command` (default `dotnet apex`).
    pub command: CommandLine,
    /// `apexTestkit.solution`, resolved from the workspace folder.
    pub solution: Option<PathBuf>,
    /// `apexTestkit.runSettingsPath`, resolved from the workspace folder.
    pub run_settings: Option<PathBuf>,
    /// `apexTestkit.buildBeforeRun` (default `true`).
    pub build_before_run: bool,
    /// `apexTestkit.configuration` (default `Debug`).
    pub configuration: Configuration,
    /// Whether the file sets any `apexTestkit.*` key.
    pub declared: bool,
}

impl ApexSettings {
    /// The defaults, as when no key is set.
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            command: CommandLine {
                program: "dotnet".to_owned(),
                prefix: vec!["apex".to_owned()],
            },
            solution: None,
            run_settings: None,
            build_before_run: true,
            configuration: Configuration::debug(),
            declared: false,
        }
    }

    /// Read `<workspace>/.vscode/settings.json`; the defaults when absent.
    ///
    /// # Errors
    /// [`SettingsError`] when the file cannot be read, is not JSONC, or a
    /// key has the wrong type.
    pub fn load(workspace: &Path) -> Result<Self, SettingsError> {
        let path: PathBuf = workspace.join(SETTINGS_FILE);
        if !path.is_file() {
            return Ok(Self::defaults());
        }
        let text: String = std::fs::read_to_string(&path).map_err(|source: std::io::Error| {
            SettingsError::Read {
                path: path.clone(),
                source,
            }
        })?;
        Self::parse(&text, workspace).map_err(|e: ParseFailure| match e {
            ParseFailure::Json(detail) => SettingsError::Json { path, detail },
            ParseFailure::Value(bad) => bad,
        })
    }

    /// Read settings text; relative paths resolve from `workspace`.
    ///
    /// # Errors
    /// Not JSONC, or a key has the wrong type.
    pub fn parse(text: &str, workspace: &Path) -> Result<Self, ParseFailure> {
        let value: Value = serde_json::from_str(&strip_jsonc(text))
            .map_err(|e: serde_json::Error| ParseFailure::Json(e.to_string()))?;
        let empty: Map<String, Value> = Map::new();
        let map: &Map<String, Value> = value.as_object().unwrap_or(&empty);
        let mut found: Self = Self::defaults();
        found.declared = map.keys().any(|k: &String| k.starts_with("apexTestkit."));
        if let Some(text) = raw_text(map, "apexTestkit.command")? {
            found.command =
                CommandLine::parse(text).ok_or(bad("apexTestkit.command", "a command"))?;
        }
        if let Some(text) = raw_text(map, "apexTestkit.configuration")? {
            found.configuration =
                Configuration::new(text).ok_or(bad("apexTestkit.configuration", "a name"))?;
        }
        found.solution = text_of(map, "apexTestkit.solution")?.map(|p: &str| workspace.join(p));
        found.run_settings =
            text_of(map, "apexTestkit.runSettingsPath")?.map(|p: &str| workspace.join(p));
        if let Some(flag) = map.get("apexTestkit.buildBeforeRun") {
            found.build_before_run = flag
                .as_bool()
                .ok_or(bad("apexTestkit.buildBeforeRun", "true or false"))?;
        }
        Ok(found)
    }
}

/// Why settings text was refused.
#[derive(Debug, thiserror::Error)]
pub enum ParseFailure {
    /// Not JSONC.
    #[error("not JSON: {0}")]
    Json(String),
    /// A key with the wrong type.
    #[error(transparent)]
    Value(SettingsError),
}

fn bad(key: &'static str, wants: &'static str) -> ParseFailure {
    ParseFailure::Value(SettingsError::BadValue { key, wants })
}

/// A string key as written. Command and configuration have non-empty
/// defaults, so a blank value set on purpose is refused, not defaulted.
fn raw_text<'a>(
    map: &'a Map<String, Value>,
    key: &'static str,
) -> Result<Option<&'a str>, ParseFailure> {
    map.get(key)
        .map(|v: &Value| v.as_str().ok_or(bad(key, "a string")))
        .transpose()
}

/// A path key; empty counts as unset (the extension's default is empty).
fn text_of<'a>(
    map: &'a Map<String, Value>,
    key: &'static str,
) -> Result<Option<&'a str>, ParseFailure> {
    match map.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_str()
            .map(|t: &str| Some(t.trim()).filter(|t: &&str| !t.is_empty()))
            .ok_or(bad(key, "a string")),
    }
}
