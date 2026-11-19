//! The language tools in real windows: Rust (BBEOD27) and Python (BBEOD28);
//! both on the real servers by mouse and keys (BBEOD29, `live`).

pub(crate) mod live;
mod python;
#[path = "bb_rust/mod.rs"]
mod rust;
