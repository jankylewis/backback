//! The Dock icon (BB-FEAT-080). macOS draws an app's bundle icon on a
//! rounded tile; an image set at launch with `setApplicationIconImage` is
//! drawn as it is, so the pixel mascot shows with no tile, as ARM, fleetmic
//! and nhetrinh do. The one `unsafe` call of backback lives here (Peaker
//! 2026-11-16); every other crate forbids `unsafe`.

mod bb_icon;

pub use bb_icon::{set_icon, DockError};

#[cfg(test)]
mod tests;
