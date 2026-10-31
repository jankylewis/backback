mod bb_tabs;
mod input;
mod prompt;
mod view;

pub(crate) use bb_tabs::Tabs;
pub(crate) use input::handle_key;
pub(crate) use view::{editor_body, tab_bar};
