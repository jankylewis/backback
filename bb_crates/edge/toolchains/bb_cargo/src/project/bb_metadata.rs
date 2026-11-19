//! The Cargo workspace's members, from `cargo metadata --no-deps
//! --format-version 1` (no dependency is resolved or fetched).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde::Deserialize;

use crate::run::bb_proc::{self, Output};
use crate::CargoError;

/// How long `cargo metadata` may take.
pub const METADATA_DEADLINE: Duration = Duration::from_secs(60);

/// One workspace member.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member {
    /// The package name.
    pub name: String,
    /// Its `Cargo.toml`.
    pub manifest: PathBuf,
}

impl Member {
    /// The folder of its `Cargo.toml`.
    #[must_use]
    pub fn dir(&self) -> &Path {
        self.manifest.parent().unwrap_or(&self.manifest)
    }
}

/// The workspace: its root and members, by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoWorkspace {
    /// `workspace_root`.
    pub root: PathBuf,
    /// The members, sorted by name.
    pub members: Vec<Member>,
}

#[derive(Deserialize)]
struct Raw {
    packages: Vec<RawPackage>,
    workspace_members: Vec<String>,
    workspace_root: PathBuf,
}

#[derive(Deserialize)]
struct RawPackage {
    id: String,
    name: String,
    manifest_path: PathBuf,
}

/// Read the members of the workspace at `root` (a folder with a
/// `Cargo.toml`) within `deadline`.
///
/// # Errors
/// The tool cannot run or passes its deadline, exits non-zero
/// ([`CargoError::Failed`]), or prints JSON this reader does not know.
pub fn members(root: &Path, deadline: Duration) -> Result<CargoWorkspace, CargoError> {
    let mut command: Command = Command::new("cargo");
    command
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--color",
            "never",
        ])
        .current_dir(root);
    let done: Output = bb_proc::run(command, "cargo metadata", deadline, None)?;
    if done.code != Some(0) {
        // The message and its place lead cargo's stderr (BBEOD33).
        return Err(CargoError::Failed {
            what: String::from("cargo metadata"),
            code: done.code,
            detail: super::failure::kept(&done.stderr, 12),
        });
    }
    parse(&done.stdout)
}

/// The workspace in `cargo metadata` JSON.
///
/// # Errors
/// [`CargoError::Json`].
pub fn parse(json: &str) -> Result<CargoWorkspace, CargoError> {
    let raw: Raw = serde_json::from_str(json)?;
    let mut members: Vec<Member> = raw
        .packages
        .into_iter()
        .filter(|p: &RawPackage| raw.workspace_members.contains(&p.id))
        .map(|p: RawPackage| Member {
            name: p.name,
            manifest: p.manifest_path,
        })
        .collect();
    members.sort();
    Ok(CargoWorkspace {
        root: raw.workspace_root,
        members,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_come_sorted_and_only_from_the_workspace_list() {
        let json: &str = r#"{"packages":[
            {"id":"path+file:///w/beta#0.1.0","name":"beta","manifest_path":"/w/beta/Cargo.toml","targets":[]},
            {"id":"path+file:///w/alpha#0.1.0","name":"alpha","manifest_path":"/w/alpha/Cargo.toml"},
            {"id":"other","name":"zeta","manifest_path":"/x/Cargo.toml"}],
            "workspace_members":["path+file:///w/alpha#0.1.0","path+file:///w/beta#0.1.0"],
            "workspace_root":"/w","version":1}"#;
        let found: CargoWorkspace = parse(json).unwrap();
        let names: Vec<&str> = found
            .members
            .iter()
            .map(|m: &Member| m.name.as_str())
            .collect();
        assert_eq!(names, ["alpha", "beta"]);
        assert_eq!(found.root, PathBuf::from("/w"));
        assert_eq!(found.members.first().unwrap().dir(), Path::new("/w/alpha"));
    }

    #[test]
    fn json_this_reader_does_not_know_is_a_typed_error() {
        assert!(matches!(
            parse("{\"packages\":1}"),
            Err(CargoError::Json(_))
        ));
    }
}
