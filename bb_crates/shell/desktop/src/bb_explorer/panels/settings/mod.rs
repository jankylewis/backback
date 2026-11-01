//! Settings (BBEOD08): one font and size for every surface, the tool window
//! side, the Settings window; zoom (BBEOD14).

mod bb_store;
pub(crate) mod keys;
pub(crate) mod view;

#[cfg(test)]
pub(crate) use bb_store::apply;
pub(crate) use bb_store::{install, root, typography};
pub(crate) use keys::{
    bindings as zoom_bindings, menu_items as zoom_menu_items, register as register_zoom,
    settings_item,
};

#[cfg(test)]
mod tests;
