//! MSBuild's diagnostic lines: `path(line,col): error CODE: message [project]`.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Severity {
    Error,
    Warning,
}

/// One build error or warning, at a 1-based line and column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Issue {
    pub(crate) path: PathBuf,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) severity: Severity,
    /// `CS0103: The name 'x' does not exist in the current context`.
    pub(crate) text: String,
}

/// Every error and warning in `output`, once each (MSBuild repeats them in
/// its summary), errors first. A relative path is joined to `root`.
pub(crate) fn issues(output: &str, root: &Path) -> Vec<Issue> {
    let mut out: Vec<Issue> = Vec::new();
    for line in output.lines() {
        if let Some(issue) = parse(line.trim(), root) {
            if !out.contains(&issue) {
                out.push(issue);
            }
        }
    }
    out.sort_by_key(|issue: &Issue| issue.severity == Severity::Warning);
    out
}

fn parse(line: &str, root: &Path) -> Option<Issue> {
    let (place, rest): (&str, &str) = line.split_once("): ")?;
    let (path, position): (&str, &str) = place.rsplit_once('(')?;
    let mut numbers: std::str::Split<'_, char> = position.split(',');
    let line_no: u32 = numbers.next()?.trim().parse().ok()?;
    let column: u32 = numbers
        .next()
        .map_or(Some(1), |c: &str| c.trim().parse().ok())?;
    let (severity, text): (Severity, &str) = match rest.strip_prefix("error ") {
        Some(text) => (Severity::Error, text),
        None => (Severity::Warning, rest.strip_prefix("warning ")?),
    };
    let text: &str = text
        .rsplit_once(" [")
        .map_or(text, |(message, _): (&str, &str)| message);
    let path: PathBuf = PathBuf::from(path.trim());
    Some(Issue {
        path: if path.is_absolute() {
            path
        } else {
            root.join(path)
        },
        line: line_no,
        column,
        severity,
        text: text.trim().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUTPUT: &str = "\
  Core -> /s/Core/bin/Debug/net10.0/Core.dll
/s/Core/Greeter.cs(7,9): error CS0103: The name 'x' does not exist in the current context [/s/Core/Core.csproj]
/s/Core/Greeter.cs(3,14): warning CS8618: Non-nullable field 'name' is uninitialized. [/s/Core/Core.csproj]
Build FAILED.
/s/Core/Greeter.cs(7,9): error CS0103: The name 'x' does not exist in the current context [/s/Core/Core.csproj]
Specs/A.cs(2,1): error CS1002: ; expected [/s/Specs/Specs.csproj]
    1 Warning(s)
    2 Error(s)";

    #[test]
    fn each_issue_once_errors_first_with_its_place_and_code() {
        let found: Vec<Issue> = issues(OUTPUT, Path::new("/s"));
        assert_eq!(found.len(), 3, "{found:?}");
        assert_eq!(found[0].path, PathBuf::from("/s/Core/Greeter.cs"));
        assert_eq!((found[0].line, found[0].column), (7, 9));
        assert_eq!(
            found[0].text,
            "CS0103: The name 'x' does not exist in the current context"
        );
        assert_eq!(
            found[1].path,
            PathBuf::from("/s/Specs/A.cs"),
            "relative paths join the root"
        );
        assert_eq!(found[2].severity, Severity::Warning);
    }

    #[test]
    fn lines_that_are_not_diagnostics_are_ignored() {
        let none: Vec<Issue> = issues(
            "Build succeeded.\n    0 Warning(s)\n  Core -> /s/x.dll\n",
            Path::new("/s"),
        );
        assert_eq!(none, Vec::<Issue>::new());
    }
}
