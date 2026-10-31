//! Test runner facade over `dotnet test`.
//!
//! .NET has two test platforms: VSTest (legacy) and Microsoft.Testing.Platform
//! (MTP, opt-in via `global.json` `"runner": "Microsoft.Testing.Platform"` on
//! .NET 10). Their discovery output differs, so the runner is an enum and the
//! parser is chosen by variant. M2 fills the MTP JSON path (`--list-tests json`).

use std::time::Duration;

use crate::bb_project::{Capability, ProjectInfo};
use crate::sdk::{Sdk, SdkError};
use crate::solution::ProjectPath;

/// Which test platform a project uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestPlatform {
    /// Legacy VSTest (`dotnet test --list-tests` prints a text list).
    VsTest,
    /// Microsoft.Testing.Platform (`--list-tests json` is being added to the SDK).
    Mtp,
}

/// A discovered test's fully qualified name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestName(String);

impl TestName {
    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Lists and (later) runs tests for one project.
#[derive(Debug, Clone)]
pub struct TestRunner {
    sdk: Sdk,
    platform: TestPlatform,
}

impl TestPlatform {
    /// The platform a detected project runs on.
    #[must_use]
    pub fn of(project: &ProjectInfo) -> Self {
        if project.has(Capability::TestingPlatform) {
            Self::Mtp
        } else {
            Self::VsTest
        }
    }
}

impl TestRunner {
    /// Build a runner for a platform.
    #[must_use]
    pub fn new(sdk: Sdk, platform: TestPlatform) -> Self {
        Self { sdk, platform }
    }

    /// Discover tests without running them.
    ///
    /// # Errors
    /// See [`Sdk::run`].
    pub async fn list(
        &self,
        project: &ProjectPath,
        timeout: Duration,
    ) -> Result<Vec<TestName>, SdkError> {
        let path: String = project.as_path().to_string_lossy().into_owned();
        let args: Vec<&str> = match self.platform {
            TestPlatform::VsTest => vec!["test", &path, "--list-tests", "--nologo"],
            TestPlatform::Mtp => vec!["test", "--project", &path, "--list-tests"],
        };
        let out: String = self.sdk.run(&args, timeout).await?;
        Ok(match self.platform {
            TestPlatform::VsTest => parse_vstest_list(&out),
            TestPlatform::Mtp => parse_mtp_text_list(&out),
        })
    }
}

/// VSTest prints "The following Tests are available:" then one indented name per line.
fn parse_vstest_list(out: &str) -> Vec<TestName> {
    out.lines()
        .skip_while(|l: &&str| !l.contains("The following Tests are available"))
        .skip(1)
        .map(str::trim)
        .filter(|l: &&str| !l.is_empty())
        .map(|l: &str| TestName(l.to_owned()))
        .collect()
}

/// MTP text output: one test per line, indented; headers and summaries are not indented.
fn parse_mtp_text_list(out: &str) -> Vec<TestName> {
    out.lines()
        .filter(|l: &&str| l.starts_with(' ') || l.starts_with('\t'))
        .map(str::trim)
        .filter(|l: &&str| !l.is_empty())
        .map(|l: &str| TestName(l.to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vstest_list_is_parsed() {
        let out: &str = "Test run for X.dll (.NETCoreApp,Version=v10.0)\nThe following Tests are available:\n    A.B.C\n    A.B.D\n";
        let names: Vec<String> = parse_vstest_list(out)
            .into_iter()
            .map(|t: TestName| t.0)
            .collect();
        assert_eq!(names, vec!["A.B.C", "A.B.D"]);
    }

    #[test]
    fn mtp_text_list_keeps_only_indented_lines() {
        let out: &str =
            "Microsoft.Testing.Platform v2\n  Ns.Class.Method\n  Ns.Class.Other\nTotal: 2\n";
        let names: Vec<String> = parse_mtp_text_list(out)
            .into_iter()
            .map(|t: TestName| t.0)
            .collect();
        assert_eq!(names, vec!["Ns.Class.Method", "Ns.Class.Other"]);
    }
}
