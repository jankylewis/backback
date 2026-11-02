//! An app opened from the Dock or Finder gets launchd's minimal `PATH`
//! (`/usr/bin:/bin:/usr/sbin:/sbin`), so `dotnet`, the .NET SDK and tools on
//! the user's shell `PATH` are not found: projects fail to load, Build, tests
//! and F5 fail (BB-BUG-009). As VS Code does ("resolve shell environment"),
//! read the login shell's `PATH` once at start and adopt it; if the shell
//! gives nothing, add the usual .NET and Homebrew folders.

use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const START: &str = "__BB_PATH__";
const END: &str = "__BB_END__";
/// How long the login shell may take (VS Code allows 10 s).
const SHELL_DEADLINE: Duration = Duration::from_secs(5);

/// True when the app was not started from a terminal (no `TERM`): its
/// `PATH` is launchd's, not the user's.
#[must_use]
pub fn opened_outside_a_terminal(term: Option<&OsStr>) -> bool {
    term.is_none_or(OsStr::is_empty)
}

/// The text between the markers the shell printed; banners and prompts an
/// interactive shell writes around it are ignored.
#[must_use]
pub fn marked_path(output: &str) -> Option<String> {
    let after: &str = output.split_once(START)?.1;
    let path: &str = after.split_once(END)?.0;
    (!path.trim().is_empty()).then(|| path.to_owned())
}

/// The folders added when the shell gives no `PATH`.
#[must_use]
pub fn fallback_dirs(home: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![
        PathBuf::from("/usr/local/share/dotnet"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/opt/homebrew/bin"),
    ];
    if let Some(home) = home {
        dirs.push(home.join(".dotnet"));
        dirs.push(home.join(".dotnet/tools"));
    }
    dirs
}

/// The login shell's `PATH`: `shell -l -i -c` prints it between markers,
/// within `deadline`; `None` when the shell fails, is silent or too slow.
#[must_use]
pub fn login_path(shell: &Path, deadline: Duration) -> Option<String> {
    let script: String = format!("printf '\\n{START}%s{END}\\n' \"$PATH\"");
    let mut child: Child = Command::new(shell)
        .args(["-l", "-i", "-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout: std::process::ChildStdout = child.stdout.take()?;
    let reader: std::thread::JoinHandle<String> = std::thread::spawn(move || {
        let mut text: String = String::new();
        let _partial: std::io::Result<usize> = stdout.read_to_string(&mut text);
        text
    });
    let started: Instant = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) | Err(_) => {
                let _gone: std::io::Result<()> = child.kill();
                let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
                break;
            }
        }
    }
    marked_path(&reader.join().ok()?)
}

/// At start, before any thread or child process: when opened outside a
/// terminal, adopt the login shell's `PATH` (or add the fallback folders).
pub fn adopt_login_path() {
    if !opened_outside_a_terminal(std::env::var_os("TERM").as_deref()) {
        return;
    }
    let shell: PathBuf =
        std::env::var_os("SHELL").map_or_else(|| PathBuf::from("/bin/zsh"), PathBuf::from);
    let path: Option<OsString> = login_path(&shell, SHELL_DEADLINE)
        .map(OsString::from)
        .or_else(with_fallback);
    if let Some(path) = path {
        std::env::set_var("PATH", path);
    }
}

/// Today's `PATH` with the fallback folders added.
fn with_fallback() -> Option<OsString> {
    let home: Option<PathBuf> = std::env::var_os("HOME").map(PathBuf::from);
    let current: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p: OsString| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    let all: Vec<PathBuf> = current
        .into_iter()
        .chain(fallback_dirs(home.as_deref()))
        .collect();
    std::env::join_paths(all).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_app_without_a_terminal_adopts_the_shell_path() {
        assert!(opened_outside_a_terminal(None));
        assert!(opened_outside_a_terminal(Some(OsStr::new(""))));
        assert!(!opened_outside_a_terminal(Some(OsStr::new(
            "xterm-256color"
        ))));
    }

    #[test]
    fn the_path_is_read_between_the_markers_whatever_the_shell_prints() {
        let noisy: String = format!("Welcome!\nlast login\n\n{START}/a/bin:/b{END}\n% ");
        assert_eq!(marked_path(&noisy).as_deref(), Some("/a/bin:/b"));
        assert_eq!(marked_path("no markers"), None);
        assert_eq!(marked_path(&format!("{START}  {END}")), None);
    }

    #[test]
    fn the_fallback_has_the_dotnet_folders() {
        let dirs: Vec<PathBuf> = fallback_dirs(Some(Path::new("/h")));
        assert!(dirs.contains(&PathBuf::from("/usr/local/share/dotnet")));
        assert!(dirs.contains(&PathBuf::from("/h/.dotnet/tools")));
    }

    #[test]
    fn a_real_login_shell_gives_its_path_and_a_missing_one_gives_nothing() {
        let path: Option<String> = login_path(Path::new("/bin/sh"), Duration::from_secs(5));
        assert!(path.is_some_and(|p: String| p.contains("/usr/bin")));
        let missing: Option<String> =
            login_path(Path::new("/no/such/shell"), Duration::from_secs(1));
        assert_eq!(missing, None);
    }
}
