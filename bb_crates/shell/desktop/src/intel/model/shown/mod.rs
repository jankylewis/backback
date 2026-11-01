//! What the window shows from the server: the load status and the
//! diagnostics of the versions on screen.

mod bb_diags;
mod status;
#[cfg(test)]
mod tests;

pub(crate) use bb_diags::{Diags, Sent};
pub(crate) use status::LoadStatus;
