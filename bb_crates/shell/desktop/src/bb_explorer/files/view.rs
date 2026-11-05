//! The Explorer row's right-click menu, drawn over the window at the click.

use bb_ui::bb_theme::Theme;
use gpui::{anchored, deferred, div, prelude::*, AnyElement, Context, Window};

use super::{apply, Item, NameOp};
use crate::{color, ExplorerView};

/// The rows: selector name, label, what it does; the root keeps only New.
const ROWS: [(&str, &str, Item); 4] = [
    ("new-file", "New File...", Item::Name(NameOp::NewFile)),
    ("new-folder", "New Folder...", Item::Name(NameOp::NewFolder)),
    ("rename", "Rename...", Item::Name(NameOp::Rename)),
    ("delete", "Delete", Item::Delete),
];

/// The open menu, if any.
pub(crate) fn menu(view: &ExplorerView, cx: &mut Context<ExplorerView>) -> Option<AnyElement> {
    let open: &super::FileMenu = view.file_menu.as_ref()?;
    let theme: Theme = view.theme;
    let shown: usize = if open.root { 2 } else { ROWS.len() };
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
                view.file_menu = None;
                cx.notify();
            },
        ))
        .children(
            ROWS.iter()
                .take(shown)
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
