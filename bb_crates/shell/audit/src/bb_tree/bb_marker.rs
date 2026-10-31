//! Backbone marker rule: the one most critical entry per folder carries a
//! marker, `_` in C# trees and `bb_` in Rust and neutral trees.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{file_name, read_dir, SKIP_DIRS};
use crate::AuditError;

/// File endings that mark C# code.
const CSHARP_EXTS: [&str; 4] = ["cs", "csproj", "slnx", "sln"];

/// What kind of code a folder holds, counting everything below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialect {
    /// Any Rust (`.rs` or `Cargo.toml`) below: Rust wins over C#.
    Rust,
    /// C# and no Rust below.
    CSharp,
    /// No code below (docs, assets).
    Neutral,
}

impl Dialect {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Rust, _) | (_, Self::Rust) => Self::Rust,
            (Self::CSharp, _) | (_, Self::CSharp) => Self::CSharp,
            (Self::Neutral, Self::Neutral) => Self::Neutral,
        }
    }

    fn of_file(path: &Path) -> Self {
        let ext: String = path
            .extension()
            .map(|e: &std::ffi::OsStr| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        if file_name(path) == "Cargo.toml" || ext == "rs" {
            Self::Rust
        } else if CSHARP_EXTS.contains(&ext.as_str()) {
            Self::CSharp
        } else {
            Self::Neutral
        }
    }

    /// The marker this dialect requires, and the one it rejects.
    fn markers(self) -> (&'static str, &'static str) {
        match self {
            Self::CSharp => ("_", "bb_"),
            Self::Rust | Self::Neutral => ("bb_", "_"),
        }
    }
}

/// The dialect of every audited folder, computed in one bottom-up pass.
pub(crate) struct Dialects {
    by_dir: HashMap<PathBuf, Dialect>,
    repo: Dialect,
}

impl Dialects {
    /// Classify `root` and every folder below it.
    pub(crate) fn scan(root: &Path) -> Result<Self, AuditError> {
        let mut by_dir: HashMap<PathBuf, Dialect> = HashMap::new();
        let repo: Dialect = classify(root, &mut by_dir)?;
        Ok(Self { by_dir, repo })
    }

    /// The folder's own dialect; a neutral folder follows the repo dialect.
    pub(crate) fn of(&self, dir: &Path) -> Dialect {
        match self.by_dir.get(dir).copied().unwrap_or(Dialect::Neutral) {
            Dialect::Neutral => self.repo,
            own @ (Dialect::Rust | Dialect::CSharp) => own,
        }
    }
}

fn classify(dir: &Path, by_dir: &mut HashMap<PathBuf, Dialect>) -> Result<Dialect, AuditError> {
    let mut kind: Dialect = Dialect::Neutral;
    for entry in read_dir(dir)? {
        let found: Dialect = if entry.is_dir() {
            if SKIP_DIRS.contains(&file_name(&entry).as_str()) {
                continue;
            }
            classify(&entry, by_dir)?
        } else {
            Dialect::of_file(&entry)
        };
        kind = kind.merge(found);
    }
    by_dir.insert(dir.to_path_buf(), kind);
    Ok(kind)
}

/// Names fixed by Cargo, Rust or the repo: they never carry the marker, never
/// count toward it, and are never renamed.
pub(crate) const FIXED_NAMES: [&str; 14] = [
    "lib.rs",
    "main.rs",
    "mod.rs",
    "build.rs",
    "Cargo.toml",
    "Cargo.lock",
    "src",
    "tests",
    "benches",
    "examples",
    "README.md",
    "LICENSE",
    "AGENTS.md",
    "CLAUDE.md",
];

/// The marker problem of one folder, if any: every folder with 2+ visible
/// entries carries exactly one marker on a non-fixed entry, unless all its
/// entries are fixed names.
pub(crate) fn problem(visible: &[String], dialect: Dialect) -> Option<String> {
    let (marker, wrong): (&str, &str) = dialect.markers();
    if let Some(bad) = visible.iter().find(|n: &&String| n.starts_with(wrong)) {
        return Some(format!(
            "`{wrong}` is not the marker here, use `{marker}`: {bad}"
        ));
    }
    let free: Vec<&String> = visible
        .iter()
        .filter(|n: &&String| !FIXED_NAMES.contains(&n.as_str()))
        .collect();
    let carried: usize = free
        .iter()
        .filter(|n: &&&String| n.starts_with(marker))
        .count();
    if carried > 1 {
        Some(format!("{carried} `{marker}` entries, max 1"))
    } else if carried == 0 && visible.len() >= 2 && !free.is_empty() {
        let names: Vec<&str> = free.iter().map(|n: &&String| n.as_str()).collect();
        Some(format!(
            "needs one `{marker}` entry among: {}",
            names.join(", ")
        ))
    } else {
        None
    }
}
