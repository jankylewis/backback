//! The tab keys, as in VS Code: Cmd+K Shift+Enter pins or unpins, Cmd+Alt+T
//! closes the others, Cmd+K Cmd+W closes all; Ctrl+Cmd+Right / Left move
//! the tab one group right / left, a new group made on the right up to
//! three (BB-BUG-045); Cmd+\\ moves it right, or left from the third group.

use gpui::{prelude::*, Context, KeyBinding, Window};

use super::super::MAX_GROUPS;
use super::menu::{apply, Item, Toward};
use super::CloseSet;
use crate::ExplorerView;

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
             cx: &mut Context<ExplorerView>| move_active(v, Some(Toward::Right), w, cx),
        ))
        .on_action(cx.listener(
            |v: &mut ExplorerView,
             _: &MoveTabLeft,
             w: &mut Window,
             cx: &mut Context<ExplorerView>| move_active(v, Some(Toward::Left), w, cx),
        ))
        .on_action(cx.listener(
            |v: &mut ExplorerView,
             _: &SplitEditor,
             w: &mut Window,
             cx: &mut Context<ExplorerView>| move_active(v, None, w, cx),
        ))
}

/// Move the active tab one group `toward`; `None` (Cmd+\\) moves it right,
/// or left from the last possible group.
fn move_active(
    view: &mut ExplorerView,
    toward: Option<Toward>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if let Some(index) = view.tabs.active_index() {
        let last: bool = view.tabs.focused().0.saturating_add(1) >= MAX_GROUPS;
        let to: Toward = toward.unwrap_or(if last { Toward::Left } else { Toward::Right });
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
