//! A group's tab bar: one item per tab, pinned ones first with a pin in
//! place of the close cross; a right-click opens the tab's menu
//! (BB-BUG-039); a tab drags to the other group (BB-BUG-040). Tabs keep
//! their width and the bar scrolls sideways; the active tab scrolls into
//! sight (BB-BUG-062).

#[cfg(test)]
mod bb_window_tests;

use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, IntoElement, MouseButton, Render, Window};

use super::super::{Group, Tab, Tabs, MAX_GROUPS};
use super::{menu, TabMenu};
use crate::{color, ExplorerView};

/// Each group's bar scroll, and the tab it last brought into sight.
#[derive(Default)]
pub(crate) struct Bars {
    scrolls: [gpui::ScrollHandle; MAX_GROUPS],
    shown: [std::cell::Cell<Option<usize>>; MAX_GROUPS],
}

impl Bars {
    /// `side`'s bar scroll; a new active tab at `active` (its place in the
    /// group) scrolls into sight once, so the wheel can move away after.
    fn scroll(&self, side: Group, active: Option<usize>) -> gpui::ScrollHandle {
        let handle: gpui::ScrollHandle = self.scrolls.get(side.0).cloned().unwrap_or_default();
        if let (Some(at), Some(shown)) = (active, self.shown.get(side.0)) {
            if shown.replace(Some(at)) != Some(at) {
                handle.scroll_to_item(at);
            }
        }
        handle
    }
}

/// A tab being dragged: its index and the name its ghost shows.
#[derive(Clone, Debug)]
pub(crate) struct DraggedTab {
    pub(crate) index: usize,
    name: String,
    theme: Theme,
}

/// The ghost under the pointer while a tab drags.
struct DragGhost(DraggedTab);

impl Render for DragGhost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme: Theme = self.0.theme;
        div()
            .px_2()
            .py_1()
            .bg(color(theme.overlay))
            .text_color(color(theme.text))
            .child(self.0.name.clone())
    }
}

pub(crate) fn tab_bar(
    tabs: &Tabs,
    side: Group,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let loading: &[std::path::PathBuf] = if side == tabs.focused() {
        tabs.loading()
    } else {
        &[]
    };
    let menu: Option<TabMenu> = tabs.menu.filter(|open: &TabMenu| {
        tabs.items()
            .get(open.index)
            .is_some_and(|tab: &Tab| tab.group == side)
    });
    let active: Option<usize> = tabs
        .group(side)
        .position(|(index, _): (usize, &Tab)| tabs.active_of(side) == Some(index));
    div()
        .id(("tab-bar", side.0))
        .debug_selector(move || format!("tab-bar-{}", side.0))
        .h(px(36.0))
        .flex()
        .overflow_x_scroll()
        .track_scroll(&tabs.bars.scroll(side, active))
        .bg(color(theme.surface))
        .children(tabs.group(side).map(|(index, tab): (usize, &Tab)| {
            tab_item(index, tab, tabs.active_of(side) == Some(index), theme, cx)
        }))
        .children(loading.iter().map(|path: &std::path::PathBuf| {
            div()
                .h_full()
                .px_2()
                .flex()
                .items_center()
                .text_color(color(theme.text_dim))
                .child(format!("Loading {}…", file_name(path)))
        }))
        .children(menu.map(|open: TabMenu| menu::tab_menu(tabs, open, theme, cx)))
}

pub(super) fn file_name(path: &std::path::Path) -> String {
    path.file_name().map_or_else(
        || String::from("Untitled"),
        |part: &std::ffi::OsStr| part.to_string_lossy().into_owned(),
    )
}

fn tab_item(
    index: usize,
    tab: &Tab,
    active: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let name: String = file_name(&tab.path);
    let dirty: &'static str = if tab.document.is_dirty() { " ●" } else { "" };
    div()
        .id(index)
        .debug_selector(move || format!("tab-{index}"))
        .h_full()
        .px_2()
        .flex()
        .items_center()
        .bg(color(if active { theme.bg } else { theme.surface }))
        .text_color(color(theme.text))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                view.tabs.switch(index);
                crate::navigation::reveal_again(view);
                window.focus(&view.editor_focus, cx);
                cx.notify();
            },
        ))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(
                move |view: &mut ExplorerView,
                      event: &gpui::MouseDownEvent,
                      _: &mut Window,
                      cx: &mut Context<ExplorerView>| {
                    view.tabs.menu = Some(TabMenu {
                        index,
                        at: event.position,
                    });
                    cx.stop_propagation();
                    cx.notify();
                },
            ),
        )
        .map(|item: gpui::Stateful<gpui::Div>| draggable(item, index, name.clone(), theme))
        .child(div().text_color(color(theme.icon_csharp)).child("C# "))
        .child(format!("{name}{dirty}"))
        .child(corner(index, tab.pinned, cx))
}

/// A tab drags to the other group; a ghost of its name follows the pointer.
fn draggable(
    item: gpui::Stateful<gpui::Div>,
    index: usize,
    name: String,
    theme: Theme,
) -> gpui::Stateful<gpui::Div> {
    item.on_drag(
        DraggedTab { index, name, theme },
        |dragged: &DraggedTab, _: gpui::Point<gpui::Pixels>, _: &mut Window, cx: &mut gpui::App| {
            cx.new(|_: &mut Context<DragGhost>| DragGhost(dragged.clone()))
        },
    )
}

/// The close cross, or for a pinned tab the pin: a click unpins it.
fn corner(index: usize, pinned: bool, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .id(format!("close-{index}"))
        .debug_selector(move || format!("close-{index}"))
        .ml_2()
        .child(if pinned { "📌" } else { "×" })
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                if pinned {
                    view.tabs.toggle_pin(index);
                } else {
                    view.tabs.close(index);
                }
                cx.notify();
            },
        ))
}
