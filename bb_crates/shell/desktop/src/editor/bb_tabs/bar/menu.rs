//! A tab's right-click menu, as in VS Code: the closes, Pin / Unpin, and
//! Move to Right / Left Group (one tab per file, so a split moves it). The
//! keys for the same live in `keys.rs`.

use bb_ui::bb_theme::Theme;
use gpui::{anchored, deferred, div, prelude::*, Context, IntoElement, Window};

use super::super::{Group, Tabs, MAX_GROUPS};
use super::{CloseSet, TabMenu};
use crate::{color, ExplorerView};

/// What one menu row does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Item {
    Close,
    Set(CloseSet),
    Pin,
    Move(Toward),
}

/// Which way a tab moves between groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Toward {
    Left,
    Right,
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
        "move-right" => return Some(Item::Move(Toward::Right)),
        "move-left" => return Some(Item::Move(Toward::Left)),
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
        Item::Move(toward) => {
            let from: Option<Group> = view
                .tabs
                .items()
                .get(index)
                .map(|tab: &super::Tab| tab.group);
            let to: Option<Group> = match toward {
                Toward::Right => from.map(Group::right),
                Toward::Left => from.and_then(Group::left),
            };
            if let Some(to) = to {
                view.tabs.move_to(index, to);
            }
        }
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
    let group: Group = tab.map_or(Group(0), |tab: &super::Tab| tab.group);
    let mut extra: Vec<(&'static str, &'static str, Item)> =
        vec![("pin", if pinned { "Unpin" } else { "Pin" }, Item::Pin)];
    if group.0.saturating_add(1) < MAX_GROUPS {
        extra.push((
            "move-right",
            "Move to Right Group",
            Item::Move(Toward::Right),
        ));
    }
    if group.left().is_some() {
        extra.push(("move-left", "Move to Left Group", Item::Move(Toward::Left)));
    }
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
            .children(ROWS.iter().chain(extra.iter()).map(
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
