//! What the window shows from the server: the load status and the
//! diagnostics of the versions on screen.

mod bb_diags;
mod side_status;
mod status;
#[cfg(test)]
mod tests;

pub(crate) use bb_diags::{Diags, Sent};
pub(crate) use side_status::{PythonIntel, RustIntel, ServerStatus};
pub(crate) use status::LoadStatus;
