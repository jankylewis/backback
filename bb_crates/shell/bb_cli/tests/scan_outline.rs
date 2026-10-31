//! `bb scan` and `bb outline` end to end on `fixtures/_Harness`.

#[cfg(test)]
mod common;

use std::path::PathBuf;

use common::{bb, harness, Run};
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
    assert!(run.out.contains("(Slnx, 4 projects)"), "{}", run.out);
    for (name, wants) in [
        ("NUnitApex", &["test exe", "VsTest", "NUnit", "Apex"][..]),
        (
            "XunitPlaywright",
            &["test dll", "Mtp", "XUnit", "Playwright"][..],
        ),
        ("McpServer", &["lib  exe", "Mcp"][..]),
        ("Core", &["lib  dll"][..]),
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
    let line: &str = line_of(&run, "NUnitApex");
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
