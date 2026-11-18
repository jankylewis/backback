//! BBEOD27 in real windows: requests reach only their language's server,
//! F12 / Cmd+click in Rust, the missing-server hint (`intel`, fake servers);
//! the Cargo explorer, Build and the Tests panel on real cargo (`panels`).

pub(super) mod bb_rig;
mod intel;
pub(super) mod panel_kit;
mod panels;
