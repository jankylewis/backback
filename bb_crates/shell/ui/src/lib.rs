//! UI contract for backback.
//!
//! The GUI framework (`GPUI`, see docs/design/DECISIONS.md) is pre-1.0, so it is kept
//! behind [`Frontend`]: core crates never depend on it, and this crate owns the
//! theme tokens so dark-by-default is a data decision, not a widget decision.

pub mod bb_theme;
pub mod typography;

pub use bb_theme::{Rgb, Theme, ThemeKind};
pub use typography::{SettingsError, Typography};

/// Frontend errors.
#[derive(Debug, thiserror::Error)]
pub enum UiError {
    /// The platform windowing or GPU layer failed.
    #[error("frontend failed: {0}")]
    Platform(String),
}

/// What a frontend needs to start. Grows per milestone; stays framework-free.
#[derive(Debug, Clone)]
pub struct Launch {
    /// Theme chosen from the user's setting or the OS preference.
    pub theme: Theme,
    /// Solution to open on start, if any.
    pub solution: Option<std::path::PathBuf>,
}

/// A frontend runs the event loop and owns the windows.
pub trait Frontend {
    /// Run until the user quits.
    ///
    /// # Errors
    /// [`UiError::Platform`] when the window or GPU layer fails.
    fn run(self, launch: Launch) -> Result<(), UiError>;
}

/// No windows: used by `bb` CLI commands and tests.
#[derive(Debug, Default, Clone, Copy)]
pub struct Headless;

impl Frontend for Headless {
    fn run(self, _launch: Launch) -> Result<(), UiError> {
        Ok(())
    }
}
