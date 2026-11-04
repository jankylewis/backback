//! A tab's right-click menu and the keys, as in VS Code: Cmd+K Shift+Enter
//! pins or unpins, Cmd+Alt+T closes the others, Cmd+K Cmd+W closes all;
//! Ctrl+Cmd+Right / Left move the tab to the right / left group and Cmd+\
//! to the other one (BB-BUG-040: one tab per file, so a split moves it).

use bb_ui::bb_theme::Theme;
use gpui::{anchored, deferred, div, prelude::*, Context, IntoElement, KeyBinding, Window};

use super::super::{Side, Tabs};
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

/// Move the active tab to the right group (made if needed).
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct MoveTabRight;

/// Move the active tab to the left group.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct MoveTabLeft;

/// Move the active tab to the other group: VS Code's Split Editor key.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct SplitEditor;

pub(crate) fn bindings() -> [KeyBinding; 6] {
    [
        KeyBinding::new("cmd-k shift-enter", PinTab, Some("Editor")),
        KeyBinding::new("cmd-alt-t", CloseOtherTabs, Some("Editor")),
        KeyBinding::new("cmd-k cmd-w", CloseAllTabs, Some("Editor")),
        KeyBinding::new("ctrl-cmd-right", MoveTabRight, Some("Editor")),
        KeyBinding::new("ctrl-cmd-left", MoveTabLeft, Some("Editor")),
        KeyBinding::new("cmd-\\", SplitEditor, Some("Editor")),
    ]
}

/// What one menu row does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Item {
    Close,
    Set(CloseSet),
    Pin,
    Move(Side),
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
    match name {
        "pin" => return Some(Item::Pin),
        "move-right" => return Some(Item::Move(Side::Right)),
        "move-left" => return Some(Item::Move(Side::Left)),
        _ => {}
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
        Item::Move(side) => view.tabs.move_to(index, side),
    }
}

pub(super) fn tab_menu(
    tabs: &Tabs,
    open: TabMenu,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let tab: Option<&super::Tab> = tabs.items().get(open.index);
    let pinned: bool = tab.is_some_and(|tab: &super::Tab| tab.pinned);
    let shift: (&'static str, &'static str, Item) = match tab.map(|tab: &super::Tab| tab.side) {
        Some(Side::Right) => ("move-left", "Move to Left Group", Item::Move(Side::Left)),
        Some(Side::Left) | None => ("move-right", "Move to Right Group", Item::Move(Side::Right)),
    };
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
            .children(ROWS.iter().chain([&pin, &shift]).map(
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
        .on_action(cx.listener(
            |v: &mut ExplorerView,
             _: &MoveTabRight,
             w: &mut Window,
             cx: &mut Context<ExplorerView>| move_active(v, Some(Side::Right), w, cx),
        ))
        .on_action(cx.listener(
            |v: &mut ExplorerView,
             _: &MoveTabLeft,
             w: &mut Window,
             cx: &mut Context<ExplorerView>| move_active(v, Some(Side::Left), w, cx),
        ))
        .on_action(cx.listener(
            |v: &mut ExplorerView,
             _: &SplitEditor,
             w: &mut Window,
             cx: &mut Context<ExplorerView>| move_active(v, None, w, cx),
        ))
}

/// Move the active tab to `side`, or to the other group for `None`.
fn move_active(
    view: &mut ExplorerView,
    side: Option<Side>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if let Some(index) = view.tabs.active_index() {
        let to: Side = side.unwrap_or_else(|| view.tabs.focused().other());
        apply(view, index, Item::Move(to));
        window.focus(&view.editor_focus, cx);
    }
    cx.notify();
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
