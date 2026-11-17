//! The window's top and bottom edges: the title bar (the folder's name and
//! Back / Forward, as Rider's) and the status bar.

mod bb_title;
pub(crate) mod status;
#[cfg(test)]
mod tests;

pub(crate) use bb_title::title_bar;
