//! BBEOD28 in real windows: Python requests reach pyright only and the
//! missing-pyright hint (`intel`, fake servers); Build > Python Check and
//! the Tests panel on this machine's tools (`panels`). BBEOD41: Python is
//! opt-in; with no saved choice it starts nothing and shows nothing
//! (`opt_in`, `unpicked`).

#[path = "bb_intel.rs"]
mod intel;
mod opt_in;
mod panels;
mod unpicked;
