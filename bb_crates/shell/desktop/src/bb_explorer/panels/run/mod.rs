//! Build (Cmd+Shift+B) and the debugger (BBEOD17).

pub(crate) mod bb_debug;
pub(crate) mod build;

/// The window-wide handlers: Go to File, Find in Files and the debugger
/// keys, so they work wherever the focus is.
pub(crate) fn root_actions(
    root: gpui::Div,
    cx: &mut gpui::Context<crate::ExplorerView>,
) -> gpui::Div {
    crate::quick_open::actions(bb_debug::actions(root, cx), cx)
}
