//! What the runners print: the Apex list JSON, FQN lists, the MTP list, and
//! the test-host process id of a debug run.

use std::path::PathBuf;

use serde::Deserialize;

use crate::model::{Fqn, FqnError};

/// Why runner output cannot be read.
#[derive(Debug, thiserror::Error)]
pub enum OutputError {
    /// The list stdout is not the one JSON object the contract names.
    #[error("list output is not {{project, assembly, specs:[{{fqn}}]}}: {0}")]
    Json(String),
    /// A listed name is not a test name.
    #[error("listed test: {0}")]
    Name(#[from] FqnError),
}

/// `apex test --list --json` stdout: exactly these three keys.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListJson {
    project: String,
    assembly: String,
    specs: Vec<SpecJson>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpecJson {
    fqn: String,
}

/// The Apex list, checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApexList {
    /// The project name.
    pub project: String,
    /// The test assembly.
    pub assembly: PathBuf,
    /// The specs in listed order.
    pub specs: Vec<Fqn>,
}

/// Read `apex test --list --json` stdout: one JSON object, nothing else.
///
/// # Errors
/// [`OutputError`] for any other shape or a spec that is not a test name.
pub fn parse_apex_list(stdout: &str) -> Result<ApexList, OutputError> {
    let raw: ListJson = serde_json::from_str(stdout.trim())
        .map_err(|e: serde_json::Error| OutputError::Json(e.to_string()))?;
    let specs: Vec<Fqn> = raw
        .specs
        .iter()
        .map(|s: &SpecJson| Fqn::new(&s.fqn))
        .collect::<Result<Vec<Fqn>, FqnError>>()?;
    Ok(ApexList {
        project: raw.project,
        assembly: PathBuf::from(raw.assembly),
        specs,
    })
}

/// One name per line (`dotnet vstest --ListFullyQualifiedTests` file);
/// blank lines are skipped.
///
/// # Errors
/// A line that is not a test name.
pub fn parse_fqn_lines(text: &str) -> Result<Vec<Fqn>, FqnError> {
    text.lines()
        .map(str::trim)
        .filter(|l: &&str| !l.is_empty())
        .map(Fqn::new)
        .collect()
}

/// The MTP app's `--list-tests` stdout: the lines after
/// `The following Tests are available:`.
///
/// # Errors
/// A listed line that is not a test name.
pub fn parse_mtp_list(stdout: &str) -> Result<Vec<Fqn>, FqnError> {
    let after: &str = stdout
        .split_once("The following Tests are available:")
        .map_or("", |(_, rest): (&str, &str)| rest);
    parse_fqn_lines(after)
}

/// A test-host process id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostPid(u32);

impl HostPid {
    /// The raw id.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

/// The host id in one output line: the Apex CLI's `{"testHostPid":N}`, or
/// VSTest's own `Process Id: N` (what `VSTEST_HOST_DEBUG=1` prints).
#[must_use]
pub fn parse_host_pid(line: &str) -> Option<HostPid> {
    let line: &str = line.trim();
    if line.starts_with('{') {
        let value: serde_json::Value = serde_json::from_str(line).ok()?;
        let pid: u64 = value.get("testHostPid")?.as_u64()?;
        return u32::try_from(pid)
            .ok()
            .filter(|p: &u32| *p > 0)
            .map(HostPid);
    }
    let (_, rest): (&str, &str) = line.split_once("Process Id:")?;
    let number: &str = rest.trim_start();
    let end: usize = number
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(number.len());
    number
        .get(..end)?
        .parse::<u32>()
        .ok()
        .filter(|p: &u32| *p > 0)
        .map(HostPid)
}
