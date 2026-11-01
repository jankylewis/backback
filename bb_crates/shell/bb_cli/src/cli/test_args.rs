//! `bb test list <sln|proj> [...]` and `bb test run <proj> [...]`.

use std::path::PathBuf;

use bb_testkit::Fqn;

use super::CliError;

/// List or run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestAction {
    /// Discover specs, never run them.
    List,
    /// Run specs and read their TRX.
    Run,
}

/// Which backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendChoice {
    /// The Apex CLI (`apexTestkit.command`).
    Apex,
    /// Plain `dotnet` (VSTest or MTP, by project).
    Dotnet,
}

/// A parsed `bb test` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestArgs {
    /// List or run.
    pub action: TestAction,
    /// A solution or project (run: a project).
    pub target: PathBuf,
    /// Forced backend; `None` picks by project and settings.
    pub backend: Option<BackendChoice>,
    /// Apex command prefix instead of `apexTestkit.command`.
    pub apex_command: Option<String>,
    /// Build configuration instead of the settings' one.
    pub configuration: Option<String>,
    /// Run only these specs.
    pub specs: Vec<Fqn>,
    /// Run only this category.
    pub category: Option<String>,
    /// A runsettings file for the run.
    pub settings: Option<PathBuf>,
    /// Build before the run.
    pub build: bool,
}

/// Parse the words after `test`.
///
/// # Errors
/// [`CliError`] for a missing action or path, an unknown flag, a flag
/// without its value, a spec that is not a test name, or run-only flags on
/// `list`.
pub fn parse(rest: &[String]) -> Result<TestArgs, CliError> {
    let (action, target, flags): (TestAction, PathBuf, &[String]) = match rest {
        [word, path, flags @ ..] if word == "list" => {
            (TestAction::List, PathBuf::from(path), flags)
        }
        [word, path, flags @ ..] if word == "run" => (TestAction::Run, PathBuf::from(path), flags),
        [word, ..] if word != "list" && word != "run" => {
            return Err(CliError::Unknown(format!("test {word}")))
        }
        _ => return Err(CliError::NeedsPath("test list|run")),
    };
    let mut args: TestArgs = TestArgs {
        action,
        target,
        backend: None,
        apex_command: None,
        configuration: None,
        specs: Vec::new(),
        category: None,
        settings: None,
        build: false,
    };
    let mut it: std::slice::Iter<'_, String> = flags.iter();
    while let Some(flag) = it.next() {
        if flag == "--build" {
            args.build = true;
            continue;
        }
        let value: &String = it.next().ok_or_else(|| CliError::Extra(flag.clone()))?;
        apply(&mut args, flag, value)?;
    }
    let run_only: bool =
        !args.specs.is_empty() || args.category.is_some() || args.settings.is_some() || args.build;
    if action == TestAction::List && run_only {
        return Err(CliError::Extra(
            "--specs/--category/--settings/--build on list".to_owned(),
        ));
    }
    Ok(args)
}

fn apply(args: &mut TestArgs, flag: &str, value: &str) -> Result<(), CliError> {
    match flag {
        "--backend" => {
            args.backend = Some(match value {
                "apex" => BackendChoice::Apex,
                "dotnet" => BackendChoice::Dotnet,
                _ => return Err(CliError::Extra(format!("--backend {value}"))),
            });
        }
        "--apex-command" => args.apex_command = Some(value.to_owned()),
        "-c" | "--configuration" => args.configuration = Some(value.to_owned()),
        "--specs" => {
            for name in split_specs(value) {
                args.specs.push(
                    Fqn::new(name)
                        .map_err(|e: bb_testkit::FqnError| CliError::BadSpec(e.to_string()))?,
                );
            }
        }
        "--category" => args.category = Some(value.to_owned()),
        "--settings" => args.settings = Some(PathBuf::from(value)),
        _ => return Err(CliError::Extra(flag.to_owned())),
    }
    Ok(())
}

/// Split on commas outside parentheses, as the Apex CLI splits `--specs`.
fn split_specs(raw: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = Vec::new();
    let mut depth: usize = 0;
    let mut start: usize = 0;
    for (i, c) in raw.char_indices() {
        match c {
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(raw.get(start..i).unwrap_or_default());
                start = i.saturating_add(1);
            }
            _ => {}
        }
    }
    parts.push(raw.get(start..).unwrap_or_default());
    parts
        .into_iter()
        .map(str::trim)
        .filter(|p: &&str| !p.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::{parse, CliError, Command};
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s: &&str| (*s).to_owned()).collect()
    }
    #[test]
    fn test_run_splits_specs_outside_parentheses_and_reads_every_flag() {
        let parsed: Command = parse(&args(&[
            "test",
            "run",
            "P.csproj",
            "--specs",
            "N.C.A,N.C.G(\"x,y\", 2) , N.C.B",
            "--category",
            "Smoke",
            "--backend",
            "apex",
            "--apex-command",
            "dotnet apex",
            "-c",
            "Release",
            "--settings",
            "s.runsettings",
            "--build",
        ]))
        .unwrap();
        let Command::Test(t): Command = parsed else {
            panic!("{parsed:?}")
        };
        let specs: Vec<&str> = t.specs.iter().map(bb_testkit::Fqn::as_str).collect();
        assert_eq!(specs, ["N.C.A", "N.C.G(\"x,y\", 2)", "N.C.B"]);
        assert_eq!(t.action, TestAction::Run);
        assert_eq!(t.backend, Some(BackendChoice::Apex));
        assert_eq!(t.apex_command.as_deref(), Some("dotnet apex"));
        assert_eq!(t.configuration.as_deref(), Some("Release"));
        assert_eq!(t.category.as_deref(), Some("Smoke"));
        assert!(t.build && t.settings == Some(PathBuf::from("s.runsettings")));
    }

    #[test]
    fn test_rejects_bad_words_flags_and_names() {
        let bad: fn(&[&str]) -> CliError = |words: &[&str]| parse(&args(words)).unwrap_err();
        assert_eq!(bad(&["test"]), CliError::NeedsPath("test list|run"));
        assert_eq!(bad(&["test", "list"]), CliError::NeedsPath("test list|run"));
        assert_eq!(
            bad(&["test", "walk", "P"]),
            CliError::Unknown("test walk".to_owned())
        );
        assert_eq!(
            bad(&["test", "run", "P", "--fast", "1"]),
            CliError::Extra("--fast".to_owned())
        );
        assert_eq!(
            bad(&["test", "run", "P", "--specs"]),
            CliError::Extra("--specs".to_owned())
        );
        assert_eq!(
            bad(&["test", "run", "P", "--backend", "x"]),
            CliError::Extra("--backend x".to_owned())
        );
        assert!(matches!(
            bad(&["test", "run", "P", "--specs", "NoClass"]),
            CliError::BadSpec(_)
        ));
        assert!(matches!(
            bad(&["test", "list", "S.slnx", "--build"]),
            CliError::Extra(_)
        ));
        let listed: Command = parse(&args(&["test", "list", "S.slnx", "-c", "Debug"])).unwrap();
        assert!(matches!(
            listed,
            Command::Test(TestArgs {
                action: TestAction::List,
                ..
            })
        ));
    }
}
