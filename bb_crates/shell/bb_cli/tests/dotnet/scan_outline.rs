//! `bb scan` and `bb outline` end to end on the desktop shell's C# fixture.

use std::path::PathBuf;

use crate::common;
use crate::common::{bb, harness, Run};
use scan_run::{line_of, scan};

#[cfg(test)]
mod scan_run {
    use std::path::PathBuf;

    use crate::common::{bb, harness, Run};

    /// `bb scan` on the harness; it must succeed.
    pub fn scan() -> Run {
        let sln: PathBuf = harness().join("Harness.slnx");
        let run: Run = bb(&["scan", sln.to_str().unwrap()]);
        assert_eq!(run.code, Some(0), "{}", run.all());
        run
    }

    /// The project line of `name`, without its leading indent.
    pub fn line_of<'a>(run: &'a Run, name: &str) -> &'a str {
        run.out
            .lines()
            .map(str::trim_start)
            .find(|l: &&str| l.starts_with(&format!("{name} ")))
            .unwrap_or_else(|| panic!("no line for {name}:\n{}", run.out))
    }
}

#[test]
fn scan_lists_capabilities_per_project() {
    let run: Run = scan();
    assert!(run.out.contains("(Slnx, 3 projects)"), "{}", run.out);
    for (name, wants) in [
        ("_NUnitApex", &["test exe", "VsTest", "NUnit", "Apex"][..]),
        (
            "_XunitPlaywright",
            &["test exe", "Mtp", "XUnit", "Playwright"][..],
        ),
        ("_Core", &["lib  dll"][..]),
    ] {
        let line: &str = line_of(&run, name);
        for want in wants {
            assert!(line.contains(want), "{name} lacks {want}: {line}");
        }
    }
}

#[test]
fn scan_counts_secrets_shows_unset_and_never_prints_a_value() {
    let settings: String =
        std::fs::read_to_string(harness().join("_NUnitApex/harness.runsettings")).unwrap();
    let secret: &str = settings
        .split_once("<HARNESS_PASSWORD>")
        .and_then(|(_, rest): (&str, &str)| rest.split_once('<'))
        .map(|(value, _): (&str, &str)| value)
        .unwrap();
    assert!(!secret.is_empty(), "fixture must carry a secret value");
    let run: Run = scan();
    assert!(!run.all().contains(secret), "secret value printed");
    let line: &str = line_of(&run, "_NUnitApex");
    assert!(line.contains("3 vars (1 secret"), "{line}");
    assert!(line.contains("UNSET HARNESS_TOTP_KEY"), "{line}");
}

#[test]
fn outline_prints_namespace_type_and_member() {
    let file: PathBuf = harness().join("Core/Greeter.cs");
    let run: Run = bb(&["outline", file.to_str().unwrap()]);
    assert_eq!(run.code, Some(0), "{}", run.all());
    for want in [
        "Namespace Harness.Core (line 1)",
        "Type Greeter (line 3)",
        "  Member Hello (line 5)",
    ] {
        assert!(run.out.contains(want), "missing {want:?}:\n{}", run.out);
    }
}

#[cfg(test)]
mod missing_project {
    use std::path::PathBuf;

    /// A valid `.slnx` with one real and one missing project, under temp.
    pub fn solution(tag: &str) -> PathBuf {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("bb-e2e-missing-{tag}-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Here")).unwrap();
        std::fs::write(
            dir.join("Here/Here.csproj"),
            "<Project Sdk=\"Microsoft.NET.Sdk\" />\n",
        )
        .unwrap();
        let sln: PathBuf = dir.join("A.slnx");
        std::fs::write(
            &sln,
            "<Solution>\n  <Project Path=\"Gone/Gone.csproj\" />\n  <Project Path=\"Here/Here.csproj\" />\n</Solution>\n",
        )
        .unwrap();
        sln
    }
}

#[test]
fn scan_with_a_missing_project_exits_non_zero_and_names_it() {
    let sln: PathBuf = missing_project::solution("scan");
    let run: Run = bb(&["scan", sln.to_str().unwrap()]);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(sln.parent().unwrap());
    assert_eq!(run.code, Some(1), "{}", run.all());
    assert!(run.err.contains("Gone.csproj"), "{}", run.all());
    assert!(
        run.err.contains("1 of 2 projects could not be read"),
        "{}",
        run.all()
    );
    assert!(
        run.out.contains("Here "),
        "the readable project is still listed: {}",
        run.out
    );
}

#[test]
fn bench_with_a_missing_project_exits_non_zero_and_names_it() {
    let sln: PathBuf = missing_project::solution("bench");
    let run: Run = bb(&["bench", sln.to_str().unwrap()]);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(sln.parent().unwrap());
    assert_eq!(run.code, Some(1), "{}", run.all());
    assert!(run.err.contains("Gone.csproj"), "{}", run.all());
}

/// Headmate ruling 2026-11-01: the real `bb` binary has no MCP entry point,
/// in its usage text or as a command, and the fixture has no MCP project.
#[test]
fn bb_has_no_mcp_entry_point() {
    let solution: String = common::harness().join("Harness.slnx").display().to_string();
    let run: common::Run = common::bb(&["mcp", "list", &solution]);
    assert_eq!(run.code, Some(2), "{}", run.all());
    assert!(run.err.contains("unknown command: mcp"), "{}", run.err);
    let usage: common::Run = common::bb(&[]);
    assert!(
        !usage.all().to_lowercase().contains("mcp"),
        "{}",
        usage.all()
    );
    let scan: common::Run = common::bb(&["scan", &solution]);
    assert!(!scan.all().to_lowercase().contains("mcp"), "{}", scan.all());
}
