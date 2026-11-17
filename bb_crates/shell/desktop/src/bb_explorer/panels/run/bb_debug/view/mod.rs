//! The window side of the debugger: the Debug panel, and session notes
//! applied to the window; `inspect` opens variables and shows a hovered
//! name's value while paused.

mod bb_panel;
pub(crate) mod inspect;
pub(crate) mod notes;
#[cfg(test)]
mod tests;

pub(crate) use bb_panel::{panel, VarTree};
