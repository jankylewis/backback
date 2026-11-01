//! Go to File (Cmd+Shift+O): a filter box over the root's files.

mod bb_model;
mod view;

pub(crate) use bb_model::QuickOpen;
pub(crate) use view::{overlay, show, GoToFile};
