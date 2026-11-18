//! The state of each server next to Roslyn as the status bar shows it:
//! rust-analyzer (BBEOD27) and pyright (BBEOD28), apart from Roslyn's: a
//! missing or crashed server never touches C# or the other language.

use bb_intel::Server;

use crate::intel::bb_worker::Link;

/// Where a server next to Roslyn stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ServerStatus {
    /// Not started: its language is not active, or nothing to serve.
    Off,
    /// Started; not initialized yet.
    Starting,
    /// Initialized; it answers as it indexes.
    Ready,
    /// Not installed; the text is the install hint, shown as is.
    Missing(String),
    /// Its connection closed.
    Stopped,
    /// It failed to start; the text says why.
    Failed(String),
}

/// The language and program names the status bar uses for `server`.
fn words(server: Server) -> (&'static str, &'static str) {
    match server {
        Server::Roslyn => ("C#", "roslyn-language-server"),
        Server::RustAnalyzer => ("Rust", "rust-analyzer"),
        Server::Pyright => ("Python", "pyright"),
    }
}

impl ServerStatus {
    /// The status bar text for `server`; empty while off.
    pub(crate) fn label(&self, server: Server) -> String {
        let (lang, program): (&str, &str) = words(server);
        match self {
            Self::Off => String::new(),
            Self::Starting => format!("{lang}: starting {program}"),
            Self::Ready => format!("{lang}: Ready"),
            Self::Missing(hint) => hint.clone(),
            Self::Stopped => format!("{lang} server stopped"),
            Self::Failed(reason) => format!("{lang} server failed: {reason}"),
        }
    }
}

/// The window's end of rust-analyzer: its link and status.
pub(crate) struct RustIntel {
    /// Requests for Rust files; `None` while it is not running.
    pub(crate) link: Option<Link>,
    /// Where it stands.
    pub(crate) status: ServerStatus,
    /// The Cargo explorer's members (BBEOD27).
    pub(crate) members: crate::rows::cargo::CargoMembers,
}

impl RustIntel {
    /// Not started.
    pub(crate) fn off() -> Self {
        Self {
            link: None,
            status: ServerStatus::Off,
            members: crate::rows::cargo::CargoMembers::Off,
        }
    }
}

/// The window's end of pyright (BBEOD28): its link and status.
pub(crate) struct PythonIntel {
    /// Requests for Python files; `None` while it is not running.
    pub(crate) link: Option<Link>,
    /// Where it stands.
    pub(crate) status: ServerStatus,
}

impl PythonIntel {
    /// Not started.
    pub(crate) fn off() -> Self {
        Self {
            link: None,
            status: ServerStatus::Off,
        }
    }
}

#[cfg(test)]
mod tests {
    use bb_intel::Server;

    use super::ServerStatus;

    #[test]
    fn a_missing_server_shows_its_install_hint_verbatim() {
        for hint in [
            bb_cargo::analyzer::MISSING_HINT,
            bb_python::pyright::MISSING_HINT,
        ] {
            assert_eq!(
                ServerStatus::Missing(hint.to_owned()).label(Server::Pyright),
                hint
            );
        }
        assert_eq!(ServerStatus::Off.label(Server::RustAnalyzer), "");
    }

    #[test]
    fn each_server_names_its_own_language() {
        assert_eq!(
            ServerStatus::Ready.label(Server::RustAnalyzer),
            "Rust: Ready"
        );
        assert_eq!(ServerStatus::Ready.label(Server::Pyright), "Python: Ready");
        assert_eq!(
            ServerStatus::Starting.label(Server::Pyright),
            "Python: starting pyright"
        );
    }
}
