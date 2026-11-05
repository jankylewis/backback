//! The Explorer row's right-click menu, drawn over the window at the click.

use bb_ui::bb_theme::Theme;
use gpui::{anchored, deferred, div, prelude::*, AnyElement, Context, Window};

use super::{apply, Item, NameOp};
use crate::{color, ExplorerView};

/// The rows: selector name, label, what it does.
const ROWS: [(&str, &str, Item); 8] = [
    ("new-file", "New File...", Item::Name(NameOp::NewFile)),
    ("new-folder", "New Folder...", Item::Name(NameOp::NewFolder)),
    ("terminal", "Open Terminal Here", Item::Terminal),
    ("copy", "Copy", Item::Copy),
    ("paste", "Paste", Item::Paste),
    ("copy-path", "Copy Path", Item::CopyPath),
    ("rename", "Rename...", Item::Name(NameOp::Rename)),
    ("delete", "Delete", Item::Delete),
];

/// Whether `item` is offered: the root is not copied, renamed or deleted
/// here, and Paste shows only after a Copy.
fn offered(item: Item, open: &super::FileMenu, copied: bool) -> bool {
    match item {
        Item::Name(NameOp::NewFile | NameOp::NewFolder) | Item::Terminal | Item::CopyPath => true,
        Item::Paste => copied,
        Item::Copy | Item::Name(NameOp::Rename) | Item::Delete => !open.root,
    }
}

/// The open menu, if any.
pub(crate) fn menu(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> Option<AnyElement> {
    let open: &super::FileMenu = view.file_menu.open.as_ref()?;
    let theme: Theme = view.theme;
    let copied: bool = view.file_menu.copied.is_some();
    let body: gpui::Stateful<gpui::Div> = div()
        .id("file-menu")
        .debug_selector(|| "file-menu".to_owned())
        .py_1()
        .min_w(gpui::px(180.0))
        .bg(color(theme.overlay))
        .text_color(color(theme.text))
        .border_1()
        .border_color(color(theme.border))
        .on_mouse_down_out(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::MouseDownEvent,
             _: &mut Window,
             cx: &mut Context<ExplorerView>| {
                view.file_menu.open = None;
                cx.notify();
            },
        ))
        .children(
            ROWS.iter()
                .filter(|(_, _, item): &&(&str, &str, Item)| offered(*item, open, copied))
                .map(|&spec: &(&'static str, &'static str, Item)| row(spec, theme, cx)),
        );
    Some(
        deferred(anchored().position(open.at).child(body))
            .with_priority(2)
            .into_any_element(),
    )
}

fn row(
    (key, label, item): (&'static str, &'static str, Item),
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .id(key)
        .debug_selector(move || format!("file-menu-{key}"))
        .px_3()
        .py_1()
        .cursor_pointer()
        .hover(move |style: gpui::StyleRefinement| style.bg(color(theme.surface)))
        .child(label)
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                cx.stop_propagation();
                apply(view, item, window, cx);
                cx.notify();
            },
        ))
}
