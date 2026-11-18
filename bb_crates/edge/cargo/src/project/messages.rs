//! `cargo build` / `cargo check` with `--message-format=json`: the
//! compiler's errors and warnings with their primary place.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

/// Which cargo command the Build menu runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildKind {
    /// `cargo build`.
    Build,
    /// `cargo check`.
    Check,
}

impl BuildKind {
    /// The cargo subcommand.
    #[must_use]
    pub fn verb(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Check => "check",
        }
    }
}

/// How bad a message is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// `error`.
    Error,
    /// `warning`.
    Warning,
}

/// One compiler message at its primary place (1-based line and column).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// The file, absolute.
    pub path: PathBuf,
    /// 1-based line.
    pub line: u32,
    /// 1-based column.
    pub column: u32,
    /// Error or warning.
    pub level: Level,
    /// The message text.
    pub text: String,
}

#[derive(Deserialize)]
struct Line {
    reason: String,
    message: Option<Compiler>,
}

#[derive(Deserialize)]
struct Compiler {
    level: String,
    message: String,
    #[serde(default)]
    spans: Vec<Span>,
}

#[derive(Deserialize)]
struct Span {
    file_name: PathBuf,
    line_start: u32,
    column_start: u32,
    is_primary: bool,
}

/// The command for `kind` over the whole workspace at `root`.
#[must_use]
pub fn command(root: &Path, kind: BuildKind) -> Command {
    let mut command: Command = Command::new("cargo");
    command
        .arg(kind.verb())
        .args(["--workspace", "--all-targets", "--message-format=json"])
        .args(["--color", "never"])
        .current_dir(root);
    command
}

/// Every error and warning in `stdout`, in order. Relative file names are
/// joined to `root`, the folder cargo ran in. Lines that are not JSON
/// (a build script's output) and notes or help are skipped.
#[must_use]
pub fn messages(stdout: &str, root: &Path) -> Vec<Message> {
    stdout
        .lines()
        .filter_map(|text: &str| serde_json::from_str::<Line>(text).ok())
        .filter(|line: &Line| line.reason == "compiler-message")
        .filter_map(|line: Line| line.message)
        .filter_map(|compiler: Compiler| message(compiler, root))
        .collect()
}

fn message(compiler: Compiler, root: &Path) -> Option<Message> {
    let level: Level = match compiler.level.as_str() {
        "error" => Level::Error,
        "warning" => Level::Warning,
        _ => return None,
    };
    let span: Span = compiler.spans.into_iter().find(|s: &Span| s.is_primary)?;
    Some(Message {
        path: root.join(span.file_name),
        line: span.line_start,
        column: span.column_start,
        level,
        text: compiler.message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ERROR: &str = r#"{"reason":"compiler-message","package_id":"beta","message":{"level":"error","message":"mismatched types","code":{"code":"E0308"},"spans":[{"file_name":"beta/src/lib.rs","line_start":3,"column_start":5,"is_primary":true},{"file_name":"beta/src/lib.rs","line_start":1,"column_start":1,"is_primary":false}]}}"#;
    const WARNING: &str = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable: `x`","spans":[{"file_name":"/abs/alpha/src/math.rs","line_start":7,"column_start":9,"is_primary":true}]}}"#;
    const ABORT: &str = r#"{"reason":"compiler-message","message":{"level":"error","message":"aborting due to 1 previous error","spans":[]}}"#;
    const ARTIFACT: &str = r#"{"reason":"compiler-artifact","package_id":"alpha"}"#;

    #[test]
    fn errors_and_warnings_keep_their_primary_place_and_skip_the_rest() {
        let stdout: String =
            [ERROR, "plain build script text", ARTIFACT, WARNING, ABORT].join("\n");
        let found: Vec<Message> = messages(&stdout, Path::new("/w"));
        assert_eq!(
            found,
            vec![
                Message {
                    path: PathBuf::from("/w/beta/src/lib.rs"),
                    line: 3,
                    column: 5,
                    level: Level::Error,
                    text: String::from("mismatched types"),
                },
                Message {
                    path: PathBuf::from("/abs/alpha/src/math.rs"),
                    line: 7,
                    column: 9,
                    level: Level::Warning,
                    text: String::from("unused variable: `x`"),
                },
            ]
        );
    }

    #[test]
    fn the_build_command_names_the_kind_and_asks_for_json() {
        let check: Command = command(Path::new("/w"), BuildKind::Check);
        let args: Vec<String> = check
            .get_args()
            .map(|a: &std::ffi::OsStr| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args.first().map(String::as_str), Some("check"));
        assert!(args.contains(&String::from("--message-format=json")));
        assert_eq!(check.get_current_dir(), Some(Path::new("/w")));
    }
}
