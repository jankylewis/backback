//! A tool failure in one line for a narrow place, the Explorer's Cargo
//! section and the Tests panel (BBEOD33): ``cargo metadata: Cargo.toml:1:
//! key with no value, expected `=` ``; the full text stays in the error.

use crate::CargoError;

impl CargoError {
    /// One line: the tool, the place it named (`file:line`, from its
    /// ` --> file:line:column` line) and its first `error:` message; else
    /// the first line of the message.
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Failed { what, detail, .. } => one_line(what, detail),
            Self::Spawn { .. }
            | Self::Timeout { .. }
            | Self::Stopped
            | Self::Json(_)
            | Self::AnalyzerMissing
            | Self::AnalyzerBroken { .. } => {
                let text: String = self.to_string();
                text.lines().next().unwrap_or_default().to_owned()
            }
        }
    }
}

fn one_line(what: &str, detail: &str) -> String {
    let lines: Vec<&str> = detail.lines().map(str::trim).collect();
    let message: &str = lines
        .iter()
        .find_map(|line: &&str| line.strip_prefix("error:"))
        .or_else(|| lines.iter().copied().find(|line: &&str| !line.is_empty()))
        .map_or("failed", str::trim);
    let place: Option<&str> = lines
        .iter()
        .find_map(|line: &&str| line.strip_prefix("--> "))
        .map(|at: &str| {
            at.rsplit_once(':')
                .map_or(at, |(file_line, _): (&str, &str)| file_line)
        });
    match place {
        Some(at) => format!("{what}: {at}: {message}"),
        None => format!("{what}: {message}"),
    }
}

/// The stderr kept for a metadata failure: from its first `error` line
/// (cargo puts the message and its place there; the tail can be only
/// `Caused by:` lines), at most `keep` lines; else the last `keep`.
pub(crate) fn kept(stderr: &str, keep: usize) -> String {
    let lines: Vec<&str> = stderr.lines().collect();
    let start: usize = lines
        .iter()
        .position(|line: &&str| line.starts_with("error"))
        .unwrap_or_else(|| lines.len().saturating_sub(keep));
    lines
        .iter()
        .skip(start)
        .take(keep)
        .copied()
        .collect::<Vec<&str>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY_WITH_NO_VALUE: &str =
        "error: key with no value, expected `=`\n --> Cargo.toml:1:2\n  |\n1 | x\n  |  ^\n";
    const MISSING_MEMBER: &str = "error: failed to load manifest for workspace member `/w/a`\nreferenced by workspace at `/w/Cargo.toml`\n\nCaused by:\n  failed to read `/w/a/Cargo.toml`\n\nCaused by:\n  No such file or directory (os error 2)\n";

    #[test]
    fn a_cargo_toml_error_is_one_line_with_its_file_and_line() {
        let error: CargoError = CargoError::failed("cargo metadata", Some(101), KEY_WITH_NO_VALUE);
        assert_eq!(
            error.summary(),
            "cargo metadata: Cargo.toml:1: key with no value, expected `=`"
        );
        assert!(error.to_string().contains("1 | x"), "the full text stays");
    }

    #[test]
    fn a_long_metadata_error_keeps_its_first_error_line() {
        let detail: String = kept(MISSING_MEMBER, 12);
        assert!(
            detail.starts_with("error: failed to load manifest"),
            "{detail}"
        );
        let error: CargoError = CargoError::Failed {
            what: String::from("cargo metadata"),
            code: Some(101),
            detail,
        };
        assert_eq!(
            error.summary(),
            "cargo metadata: failed to load manifest for workspace member `/w/a`"
        );
        assert_eq!(kept("a\nb\nc\n", 2), "b\nc", "no error line: the tail");
    }

    #[test]
    fn other_failures_give_their_first_line() {
        assert_eq!(CargoError::Stopped.summary(), "stopped");
        let error: CargoError = CargoError::failed("cargo metadata", Some(1), "");
        assert_eq!(error.summary(), "cargo metadata: failed");
    }
}
