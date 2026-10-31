//! `.runsettings` files: environment variables and worker counts.
//!
//! Security rule: a value whose *name* looks secret (password, token, key,
//! TOTP, PAT, connection string) is never stored, so it can never be shown,
//! logged or sent anywhere. Only its presence is recorded.

use std::path::{Path, PathBuf};

/// Name fragments that mark a variable as secret (matched per `_` segment).
const SECRET_MARKERS: [&str; 7] = [
    "PASSWORD",
    "PWD",
    "SECRET",
    "TOKEN",
    "KEY",
    "TOTP",
    "CONNECTIONSTRING",
];
/// Short markers that must equal a whole segment (`PAT` must not match `PATH`).
const SECRET_SEGMENTS: [&str; 1] = ["PAT"];
/// Placeholders that mean "not filled in yet".
const PLACEHOLDERS: [&str; 3] = ["__FILL_ME__", "TODO", "CHANGEME"];

/// What backback keeps of one variable's value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvValue {
    /// A non-secret value, kept as written.
    Plain(String),
    /// A secret: set, value deliberately not kept.
    Secret,
    /// A placeholder or empty: must be filled before tests can pass.
    Unset,
}

/// One environment variable from `<RunConfiguration><EnvironmentVariables>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    /// Variable name.
    pub name: String,
    /// Kept value or its classification.
    pub value: EnvValue,
}

/// A parsed runsettings file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSettings {
    /// The file.
    pub path: PathBuf,
    /// Environment variables in file order.
    pub env: Vec<EnvVar>,
    /// `<NUnit><NumberOfTestWorkers>`, if set.
    pub nunit_workers: Option<u32>,
    /// `<RunConfiguration><MaxCpuCount>`, if set.
    pub max_cpu_count: Option<u32>,
}

impl RunSettings {
    /// The runsettings a project uses: `RunSettingsFilePath` when set, else the
    /// single `*.runsettings` beside the project (templates are skipped).
    #[must_use]
    pub fn find(project_dir: &Path, property: Option<&str>) -> Option<Self> {
        let declared: Option<PathBuf> = property
            .map(|p: &str| p.replace("$(MSBuildProjectDirectory)", &project_dir.to_string_lossy()))
            .map(|p: String| PathBuf::from(p.replace('\\', "/")))
            .filter(|p: &PathBuf| p.is_file());
        let path: PathBuf = declared.or_else(|| single_runsettings(project_dir))?;
        let text: String = std::fs::read_to_string(&path).ok()?;
        Self::parse(path, &text)
    }

    /// Parse runsettings XML. `None` when the text is not XML.
    #[must_use]
    pub fn parse(path: PathBuf, text: &str) -> Option<Self> {
        let doc: roxmltree::Document<'_> = roxmltree::Document::parse(text).ok()?;
        let env: Vec<EnvVar> = doc
            .descendants()
            .filter(|n: &roxmltree::Node<'_, '_>| n.has_tag_name("EnvironmentVariables"))
            .flat_map(|n: roxmltree::Node<'_, '_>| n.children().filter(roxmltree::Node::is_element))
            .map(|n: roxmltree::Node<'_, '_>| {
                classify(n.tag_name().name(), n.text().unwrap_or_default())
            })
            .collect();
        Some(Self {
            path,
            env,
            nunit_workers: number(&doc, "NumberOfTestWorkers"),
            max_cpu_count: number(&doc, "MaxCpuCount"),
        })
    }

    /// Variables that still need a value.
    #[must_use]
    pub fn unset(&self) -> Vec<&str> {
        self.env
            .iter()
            .filter(|v: &&EnvVar| v.value == EnvValue::Unset)
            .map(|v: &EnvVar| v.name.as_str())
            .collect()
    }
}

/// Classify one variable. A secret's text is dropped here and never stored.
#[must_use]
pub fn classify(name: &str, raw: &str) -> EnvVar {
    let value: &str = raw.trim();
    let upper: String = name.to_ascii_uppercase();
    let kind: EnvValue = if value.is_empty() || PLACEHOLDERS.contains(&value) {
        EnvValue::Unset
    } else if is_secret_name(&upper) {
        EnvValue::Secret
    } else {
        EnvValue::Plain(value.to_owned())
    };
    EnvVar {
        name: name.to_owned(),
        value: kind,
    }
}

fn is_secret_name(upper: &str) -> bool {
    upper.split(['_', '-', '.']).any(|seg: &str| {
        SECRET_MARKERS.iter().any(|m: &&str| seg.contains(m)) || SECRET_SEGMENTS.contains(&seg)
    })
}

fn number(doc: &roxmltree::Document<'_>, tag: &str) -> Option<u32> {
    doc.descendants()
        .find(|n: &roxmltree::Node<'_, '_>| n.has_tag_name(tag))
        .and_then(|n: roxmltree::Node<'_, '_>| n.text())
        .and_then(|t: &str| t.trim().parse::<u32>().ok())
}

fn single_runsettings(dir: &Path) -> Option<PathBuf> {
    let found: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e: std::fs::DirEntry| e.path())
        .filter(|p: &PathBuf| {
            p.extension()
                .is_some_and(|e: &std::ffi::OsStr| e == "runsettings")
        })
        .collect();
    match found.as_slice() {
        [one] => Some(one.clone()),
        [] | [_, _, ..] => None,
    }
}
