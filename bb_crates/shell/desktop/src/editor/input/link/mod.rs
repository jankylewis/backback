//! Cmd + hover (BBEOD15): the identifier under the mouse is drawn as a link
//! once the C# server answers that it has a definition; Cmd+click goes to
//! it, as F12 does. Pressing or releasing Cmd without moving updates it.

mod bb_link;

pub(crate) use bb_link::{link_answer, link_move, on_editor, shown};

#[cfg(test)]
mod tests;
