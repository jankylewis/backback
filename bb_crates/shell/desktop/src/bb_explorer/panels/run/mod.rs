//! Build (Cmd+Shift+B) and the debugger (BBEOD17).

pub(crate) mod bb_debug;
pub(crate) mod build;

/// The window-wide handlers: Go to File, Find in Files and the debugger
/// keys, so they work wherever the focus is. Set Next Statement only while
/// `view` can run it.
pub(crate) fn root_actions(
    root: gpui::Div,
    view: &crate::ExplorerView,
    cx: &mut gpui::Context<crate::ExplorerView>,
) -> gpui::Div {
    let set_next: bool = bb_debug::session::control::can_set_next(view);
    crate::quick_open::actions(bb_debug::actions(root, set_next, cx), cx)
}
