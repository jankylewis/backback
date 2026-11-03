//! Exception breakpoints (BBEOD19), the two ticks of VS Code's Breakpoints
//! view: netcoredbg's `all` (every throw, caught or not) and
//! `user-unhandled` filters. Saved per workspace in the state folder, a
//! small versioned text file: a header line, then one filter id per line.

use std::path::{Path, PathBuf};

const HEADER: &str = "backback exceptions v1";
const FOLDER: &str = "exceptions";

/// Which exceptions pause the program.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ExceptionBreaks {
    pub(crate) all: bool,
    pub(crate) user_unhandled: bool,
}

/// One tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExceptionKind {
    All,
    UserUnhandled,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum ExceptionsError {
    #[error("exception ticks: unknown format {0:?}")]
    UnknownFormat(String),
    #[error("exception ticks: unknown filter {0:?}")]
    UnknownFilter(String),
}

impl ExceptionKind {
    /// The adapter's filter id.
    pub(crate) fn filter(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::UserUnhandled => "user-unhandled",
        }
    }

    /// The tick's label, as VS Code words it.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::All => "All Exceptions",
            Self::UserUnhandled => "User-Unhandled Exceptions",
        }
    }

    /// The tick's element id.
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::All => "exceptions-all",
            Self::UserUnhandled => "exceptions-user-unhandled",
        }
    }
}

impl ExceptionBreaks {
    /// Flip one tick; true when it is now on.
    pub(crate) fn toggle(&mut self, kind: ExceptionKind) -> bool {
        let tick: &mut bool = match kind {
            ExceptionKind::All => &mut self.all,
            ExceptionKind::UserUnhandled => &mut self.user_unhandled,
        };
        *tick = !*tick;
        *tick
    }

    /// The filter ids to send.
    pub(crate) fn filters(self) -> Vec<String> {
        [
            (self.all, ExceptionKind::All),
            (self.user_unhandled, ExceptionKind::UserUnhandled),
        ]
        .into_iter()
        .filter(|(on, _): &(bool, ExceptionKind)| *on)
        .map(|(_, kind): (bool, ExceptionKind)| kind.filter().to_owned())
        .collect()
    }

    /// The ticks saved for the workspace at `root`; none when unsaved.
    pub(crate) fn load(root: &Path) -> Self {
        state_dir().map_or_else(Self::default, |dir: PathBuf| Self::load_from(&dir, root))
    }

    /// Save the ticks of the workspace at `root`.
    pub(crate) fn save(self, root: &Path) {
        if let Some(dir) = state_dir() {
            self.save_in(&dir, root);
        }
    }

    /// An unknown format or filter is said, never guessed at.
    fn load_from(dir: &Path, root: &Path) -> Self {
        match std::fs::read_to_string(file(dir, root)) {
            Ok(text) => Self::decode(&text).unwrap_or_else(|error: ExceptionsError| {
                eprintln!("backback: {error}");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    fn save_in(self, dir: &Path, root: &Path) {
        let path: PathBuf = file(dir, root);
        if let Some(folder) = path.parent() {
            let _made: std::io::Result<()> = std::fs::create_dir_all(folder);
        }
        if let Err(error) = std::fs::write(&path, self.encode()) {
            eprintln!("backback: exception ticks not saved: {error}");
        }
    }

    fn encode(self) -> String {
        let mut text: String = format!("{HEADER}\n");
        for id in self.filters() {
            text.push_str(&id);
            text.push('\n');
        }
        text
    }

    fn decode(text: &str) -> Result<Self, ExceptionsError> {
        let mut lines: std::str::Lines<'_> = text.lines();
        let header: &str = lines.next().unwrap_or_default();
        if header != HEADER {
            return Err(ExceptionsError::UnknownFormat(header.to_owned()));
        }
        let mut ticks: Self = Self::default();
        for line in lines.map(str::trim).filter(|line: &&str| !line.is_empty()) {
            match line {
                "all" => ticks.all = true,
                "user-unhandled" => ticks.user_unhandled = true,
                other => return Err(ExceptionsError::UnknownFilter(other.to_owned())),
            }
        }
        Ok(ticks)
    }
}

/// Seam: unit tests never touch the real state folder.
fn state_dir() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    crate::launch::recent::state_dir()
}

/// `<state dir>/exceptions/<hash of the root path>`.
fn file(dir: &Path, root: &Path) -> PathBuf {
    dir.join(FOLDER).join(format!("{:016x}", fnv(root)))
}

/// FNV-1a over the path's bytes: one stable file name per workspace.
fn fnv(root: &Path) -> u64 {
    root.as_os_str()
        .as_encoded_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash: u64, byte: &u8| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_toggle_and_list_their_filters_in_the_adapter_order() {
        let mut ticks: ExceptionBreaks = ExceptionBreaks::default();
        assert!(ticks.filters().is_empty(), "off until asked, as VS Code");
        assert!(ticks.toggle(ExceptionKind::UserUnhandled));
        assert!(ticks.toggle(ExceptionKind::All));
        assert_eq!(ticks.filters(), ["all", "user-unhandled"]);
        assert!(!ticks.toggle(ExceptionKind::All));
        assert_eq!(ticks.filters(), ["user-unhandled"]);
    }

    #[test]
    fn saved_ticks_come_back_per_workspace_and_a_strange_file_is_refused() {
        let state: tempfile::TempDir = tempfile::tempdir().unwrap();
        let (a, b): (PathBuf, PathBuf) = (PathBuf::from("/w/A"), PathBuf::from("/w/B"));
        let ticks: ExceptionBreaks = ExceptionBreaks {
            all: true,
            user_unhandled: false,
        };
        ticks.save_in(state.path(), &a);
        assert_eq!(ExceptionBreaks::load_from(state.path(), &a), ticks);
        assert_eq!(
            ExceptionBreaks::load_from(state.path(), &b),
            ExceptionBreaks::default(),
            "another workspace has its own file"
        );
        assert_eq!(
            std::fs::read_to_string(file(state.path(), &a)).unwrap(),
            "backback exceptions v1\nall\n"
        );
        assert_eq!(
            ExceptionBreaks::decode("backback exceptions v2\nall\n"),
            Err(ExceptionsError::UnknownFormat(String::from(
                "backback exceptions v2"
            )))
        );
        assert_eq!(
            ExceptionBreaks::decode("backback exceptions v1\nnever\n"),
            Err(ExceptionsError::UnknownFilter(String::from("never")))
        );
    }
}
