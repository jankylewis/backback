//! A tab's right-click menu and the keys, as in VS Code: Cmd+K Shift+Enter
//! pins or unpins, Cmd+Alt+T closes the others, Cmd+K Cmd+W closes all.

use bb_ui::bb_theme::Theme;
use gpui::{anchored, deferred, div, prelude::*, Context, IntoElement, KeyBinding, Window};

use super::super::Tabs;
use super::{CloseSet, TabMenu};
use crate::{color, ExplorerView};

/// Pin the active tab, or unpin it.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct PinTab;

/// Close every unpinned tab but the active one.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct CloseOtherTabs;

/// Close every tab, pinned or not.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct CloseAllTabs;

pub(crate) fn bindings() -> [KeyBinding; 3] {
    [
        KeyBinding::new("cmd-k shift-enter", PinTab, Some("Editor")),
        KeyBinding::new("cmd-alt-t", CloseOtherTabs, Some("Editor")),
        KeyBinding::new("cmd-k cmd-w", CloseAllTabs, Some("Editor")),
    ]
}

/// What one menu row does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Item {
    Close,
    Set(CloseSet),
    Pin,
}

/// The rows: selector name, label, what it does.
const ROWS: [(&str, &str, Item); 6] = [
    ("close", "Close", Item::Close),
    ("close-others", "Close Others", Item::Set(CloseSet::Others)),
    (
        "close-right",
        "Close to the Right",
        Item::Set(CloseSet::Right),
    ),
    ("close-left", "Close to the Left", Item::Set(CloseSet::Left)),
    (
        "close-unpinned",
        "Close All Except Pinned",
        Item::Set(CloseSet::Unpinned),
    ),
    ("close-all", "Close All", Item::Set(CloseSet::All)),
];

/// The row a selector name stands for (the readback `tabmenu` step).
pub(crate) fn item(name: &str) -> Option<Item> {
    if name == "pin" {
        return Some(Item::Pin);
    }
    ROWS.iter()
        .find(|(key, _, _): &&(&str, &str, Item)| *key == name)
        .map(|(_, _, it): &(&str, &str, Item)| *it)
}

/// Do `item` for tab `index`, as its menu row does; the menu closes.
pub(crate) fn apply(view: &mut ExplorerView, index: usize, item: Item) {
    view.tabs.menu = None;
    match item {
        Item::Close => {
            let _asked: bool = view.tabs.close(index);
        }
        Item::Set(set) => {
            let _asked: bool = view.tabs.close_set(index, set);
        }
        Item::Pin => view.tabs.toggle_pin(index),
    }
}

pub(super) fn tab_menu(
    tabs: &Tabs,
    open: TabMenu,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let pinned: bool = tabs
        .items()
        .get(open.index)
        .is_some_and(|tab: &super::Tab| tab.pinned);
    let pin: (&'static str, &'static str, Item) =
        ("pin", if pinned { "Unpin" } else { "Pin" }, Item::Pin);
    let body: gpui::Stateful<gpui::Div> =
        div()
            .id("tab-menu")
            .debug_selector(|| "tab-menu".to_owned())
            .py_1()
            .min_w(gpui::px(200.0))
            .bg(color(theme.overlay))
            .text_color(color(theme.text))
            .on_mouse_down_out(cx.listener(
                |view: &mut ExplorerView,
                 _: &gpui::MouseDownEvent,
                 _: &mut Window,
                 cx: &mut Context<ExplorerView>| {
                    view.tabs.menu = None;
                    cx.notify();
                },
            ))
            .children(ROWS.iter().chain(std::iter::once(&pin)).map(
                |&spec: &(&'static str, &'static str, Item)| row(open.index, spec, theme, cx),
            ));
    deferred(anchored().position(open.at).child(body)).with_priority(2)
}

fn row(
    index: usize,
    (key, label, it): (&'static str, &'static str, Item),
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .id(key)
        .debug_selector(move || format!("tab-menu-{key}"))
        .px_3()
        .py_1()
        .hover(move |style: gpui::StyleRefinement| style.bg(color(theme.surface)))
        .child(label)
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  _: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                apply(view, index, it);
                cx.notify();
            },
        ))
}

/// The editor area's tab actions (the keys of [`bindings`]).
pub(crate) fn on_tab_actions(
    area: gpui::Stateful<gpui::Div>,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    area.on_action(cx.listener(ExplorerView::on_pin_tab))
        .on_action(cx.listener(ExplorerView::on_close_other_tabs))
        .on_action(cx.listener(ExplorerView::on_close_all_tabs))
}

impl ExplorerView {
    pub(crate) fn on_pin_tab(&mut self, _: &PinTab, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.tabs.active_index() {
            apply(self, index, Item::Pin);
        }
        cx.notify();
    }

    pub(crate) fn on_close_other_tabs(
        &mut self,
        _: &CloseOtherTabs,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.tabs.active_index() {
            apply(self, index, Item::Set(CloseSet::Others));
        }
        cx.notify();
    }

    pub(crate) fn on_close_all_tabs(
        &mut self,
        _: &CloseAllTabs,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        apply(
            self,
            self.tabs.active_index().unwrap_or(0),
            Item::Set(CloseSet::All),
        );
        cx.notify();
    }
}
