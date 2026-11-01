//! The .NET project commands: `bb scan`, `bb test` and `bb mcp`.

mod bb_testing;
pub(crate) mod mcp;
pub(crate) mod scan;
mod tree;

pub(crate) use bb_testing::run as test;

/// `bb test` problems.
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    /// The engine failed.
    #[error(transparent)]
    Engine(#[from] bb_testkit::TestkitError),
    /// The workspace settings cannot be used.
    #[error(transparent)]
    Settings(#[from] bb_testkit::SettingsError),
    /// A flag value is not usable.
    #[error("{0}")]
    Usage(String),
    /// Specs failed or errored (already printed).
    #[error("{failed} failed, {errored} errored")]
    Specs {
        /// Failed specs.
        failed: usize,
        /// Errored leaves (missing from TRX).
        errored: usize,
    },
    /// Some projects could not be listed (each already printed).
    #[error("{0} projects could not be listed")]
    Lists(usize),
}
