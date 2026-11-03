//! Navigation in the window: F12, Shift+F12, Cmd+., Problems, the status
//! bar (`bb_definition`), Go to Implementation and Go to Type Definition,
//! and Show Call Hierarchy (`callers`).

mod callers;
#[path = "bb_definition.rs"]
mod definition;
mod implementation;
