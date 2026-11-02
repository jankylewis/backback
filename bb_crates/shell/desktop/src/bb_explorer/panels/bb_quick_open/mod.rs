//! Go to File (Cmd+Shift+O), Go to Symbol (Cmd+F12) and the Find in Files
//! box (Cmd+Shift+F): one filter box over files, symbols or a search.

mod bb_model;
mod find;
mod view;

#[cfg(test)]
mod tests;

pub(crate) use bb_model::QuickOpen;
pub(crate) use find::FindInFiles;
pub(crate) use view::{actions, on_go_to_symbol, overlay, GoToFile, GoToSymbol};
