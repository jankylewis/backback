//! Build (Cmd+Shift+B) and the debugger (BBEOD17).

pub(crate) mod bb_debug;
pub(crate) mod build;

use gpui::InteractiveElement;

/// The window-wide handlers: Go to File, Find in Files, New File and the debugger
/// keys, so they work wherever the focus is. Set Next Statement only while
/// `view` can run it; no debugger key or menu item at all without C#
/// active (BBEOD26: no Rust or Python debugger, ever).
pub(crate) fn root_actions(
    root: gpui::Div,
    view: &crate::ExplorerView,
    cx: &mut gpui::Context<crate::ExplorerView>,
) -> gpui::Div {
    let set_next: bool = bb_debug::session::control::can_set_next(view);
    let root: gpui::Div = if view.langs.langs.debug_visible() {
        bb_debug::actions(root, set_next, cx)
    } else if view.langs.detecting {
        // BB-PRF-009: F5 before the languages are known says so.
        root.on_action(cx.listener(crate::launch::langs::debug_while_detecting))
    } else {
        root
    };
    crate::quick_open::actions(root, cx)
        .on_action(cx.listener(crate::editor::on_new_file))
        // Back / Forward from any focus, the Debug panel's too (BB-BUG-067).
        .on_action(cx.listener(crate::editor::go_back))
        .on_action(cx.listener(crate::editor::go_forward))
}
