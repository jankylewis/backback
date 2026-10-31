//! `bb probe <file.cs> <line:col> <feature> [--sln <solution>] [--repeat N]`.

use std::path::PathBuf;

use super::CliError;

/// Most requests one probe may time.
pub const MAX_REPEAT: usize = 1000;

/// The feature a probe asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Feature {
    /// Completion items.
    Completion,
    /// Hover text.
    Hover,
    /// Go to definition.
    Definition,
    /// Find references.
    References,
    /// The rename edit to this new name (never applied).
    Rename(String),
    /// Code actions (listed).
    Actions,
    /// Diagnostics of the file.
    Diagnostics,
}

impl Feature {
    fn parse(word: &str) -> Result<Self, CliError> {
        let bad: CliError = CliError::BadFeature(word.to_owned());
        if let Some(name) = word.strip_prefix("rename=") {
            return if name.is_empty() {
                Err(bad)
            } else {
                Ok(Self::Rename(name.to_owned()))
            };
        }
        match word {
            "completion" => Ok(Self::Completion),
            "hover" => Ok(Self::Hover),
            "definition" => Ok(Self::Definition),
            "references" => Ok(Self::References),
            "actions" => Ok(Self::Actions),
            "diagnostics" => Ok(Self::Diagnostics),
            _ => Err(bad),
        }
    }
}

/// A parsed probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    /// The C# file.
    pub file: PathBuf,
    /// 1-based line and 1-based char column.
    pub at: (u32, u32),
    /// What to ask.
    pub feature: Feature,
    /// The solution; `None` finds the nearest one above the file.
    pub solution: Option<PathBuf>,
    /// How many times to ask (the first is timed apart from the rest).
    pub repeat: usize,
}

/// Parse the words after `probe`.
pub(super) fn parse(rest: &[String]) -> Result<Probe, CliError> {
    let [file, at, feature, flags @ ..]: &[String] = rest else {
        return Err(CliError::NeedsPath("probe"));
    };
    let mut probe: Probe = Probe {
        file: PathBuf::from(file),
        at: point(at)?,
        feature: Feature::parse(feature)?,
        solution: None,
        repeat: 1,
    };
    let mut it: std::slice::Iter<'_, String> = flags.iter();
    while let Some(flag) = it.next() {
        match (flag.as_str(), it.next()) {
            ("--sln", Some(path)) => probe.solution = Some(PathBuf::from(path)),
            ("--repeat", Some(n)) => probe.repeat = repeat(n)?,
            ("--repeat", None) => return Err(CliError::BadRepeat(String::new())),
            _ => return Err(CliError::Extra(flag.clone())),
        }
    }
    Ok(probe)
}

/// `line:col`, both whole numbers from 1.
fn point(word: &str) -> Result<(u32, u32), CliError> {
    word.split_once(':')
        .and_then(|(line, col): (&str, &str)| Some((from_one(line)?, from_one(col)?)))
        .ok_or_else(|| CliError::BadPoint(word.to_owned()))
}

fn from_one(n: &str) -> Option<u32> {
    n.parse::<u32>().ok().filter(|v: &u32| *v >= 1)
}

fn repeat(n: &str) -> Result<usize, CliError> {
    n.parse::<usize>()
        .ok()
        .filter(|r: &usize| (1..=MAX_REPEAT).contains(r))
        .ok_or_else(|| CliError::BadRepeat(n.to_owned()))
}
