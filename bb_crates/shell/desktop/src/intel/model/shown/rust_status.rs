//! rust-analyzer's state as the status bar shows it (BBEOD27), apart from
//! Roslyn's: a missing or crashed Rust server never touches C#.

use crate::intel::bb_worker::Link;

/// Where rust-analyzer stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RustStatus {
    /// Not started: Rust is not active, or the folder has no `Cargo.toml`.
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

impl RustStatus {
    /// The status bar text; empty while off.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Off => String::new(),
            Self::Starting => String::from("Rust: starting rust-analyzer"),
            Self::Ready => String::from("Rust: Ready"),
            Self::Missing(hint) => hint.clone(),
            Self::Stopped => String::from("Rust server stopped"),
            Self::Failed(reason) => format!("Rust server failed: {reason}"),
        }
    }
}

/// The window's end of rust-analyzer: its link and status.
pub(crate) struct RustIntel {
    /// Requests for Rust files; `None` while it is not running.
    pub(crate) link: Option<Link>,
    /// Where it stands.
    pub(crate) status: RustStatus,
    /// The Cargo explorer's members (BBEOD27).
    pub(crate) members: crate::rows::cargo::CargoMembers,
}

impl RustIntel {
    /// Not started.
    pub(crate) fn off() -> Self {
        Self {
            link: None,
            status: RustStatus::Off,
            members: crate::rows::cargo::CargoMembers::Off,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RustStatus;

    #[test]
    fn the_missing_server_shows_its_install_hint_verbatim() {
        let hint: &str = bb_cargo::analyzer::MISSING_HINT;
        assert_eq!(RustStatus::Missing(hint.to_owned()).label(), hint);
        assert_eq!(RustStatus::Off.label(), "");
    }
}
