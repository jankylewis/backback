//! Text core for backback.
//!
//! One rope-backed [`Buffer`] per open file. Positions are newtypes so a char
//! index, a line index and an LSP (line, UTF-16 column) point can never be
//! mixed by accident; every conversion is a named, checked function.

mod buffer;
mod position;

pub use buffer::{Buffer, TextError};
pub use position::{CharIdx, LineIdx, LspPoint};
