//! The start page (BBEOD23), as Rider's Welcome screen: shown on a Finder or
//! Dock launch with no argument and by File > Start Page.... It lists the
//! recent workspaces, newest first, then Open.... A click, or Up/Down and
//! Enter, opens an entry as a command-line argument would, then the page
//! closes.

use std::path::PathBuf;

use gpui::{
    div, prelude::*, px, AnyWindowHandle, App, ClickEvent, Context, FocusHandle, KeyDownEvent,
    Subscription, Window, WindowHandle,
};

mod bb_window;
pub(super) mod entries;
pub(super) mod script;
mod ticks;

#[cfg(test)]
pub(crate) use bb_window::open;
pub(crate) use bb_window::{reopen, show};

use crate::editor::Quit;
use crate::{color, ExplorerView};
use entries::{empty_note, entries, header, label, row_label, row_texts, rows_box, StartEntry};

pub(crate) struct StartPage {
    entries: Vec<StartEntry>,
    selected: usize,
    no_write: bool,
    /// The language boxes (BBEOD26) and whether a click changed them.
    ticks: bb_lang::LangSet,
    touched: bool,
    area: ticks::Area,
    /// File > Languages...: the workspace window the next open replaces.
    replace: Option<AnyWindowHandle>,
    home: Option<PathBuf>,
    focus: FocusHandle,
    _appearance: Subscription,
}

impl StartPage {
    /// The page for `recent`, newest first; folders gone from disk are left out.
    pub(crate) fn new(
        recent: Vec<PathBuf>,
        no_write: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let entries: Vec<StartEntry> = entries(recent);
        Self {
            ticks: ticks::ticks_for(entries.first()),
            entries,
            selected: 0,
            no_write,
            touched: false,
            area: ticks::Area::Rows,
            replace: None,
            home: std::env::var_os("HOME").map(PathBuf::from),
            focus: cx.focus_handle(),
            _appearance: cx.observe_window_appearance(
                window,
                |_: &mut Self, _: &mut Window, cx: &mut Context<Self>| cx.notify(),
            ),
        }
    }

    pub(crate) fn selected(&self) -> usize {
        self.selected
    }

    /// Each entry as `state` prints it: the name and the folder, or Open....
    pub(crate) fn labels(&self) -> Vec<String> {
        self.entries.iter().map(label).collect()
    }

    fn step(&mut self, down: bool) {
        let last: usize = self.entries.len().saturating_sub(1);
        self.selected = if down {
            self.selected.saturating_add(1).min(last)
        } else {
            self.selected.saturating_sub(1)
        };
        self.refresh_ticks();
    }

    /// Open entry `index`: a workspace in its own window (the page closes), or
    /// the open panel (the page closes once a pick opens).
    /// Nothing opens while no language box is ticked (BBEOD26).
    fn choose(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected != index {
            self.selected = index;
            self.refresh_ticks();
        }
        if self.ticks.is_empty() {
            cx.notify();
            return;
        }
        match self.entries.get(index).cloned() {
            Some(StartEntry::Recent(path)) => {
                let mut start: crate::args::Start = crate::args::start_for(path, self.no_write);
                start.ticked = self.chosen(false);
                if let Some(old) = self.replace.take() {
                    bb_window::close(old, cx);
                }
                if crate::launch::open_workspace(&start, cx) {
                    window.remove_window();
                }
            }
            Some(StartEntry::Open) => self.ask(window, cx),
            None => {}
        }
        cx.notify();
    }

    fn ask(&self, window: &Window, cx: &mut App) {
        let page: Option<AnyWindowHandle> = Some(window.window_handle());
        super::ask(cx, (self.no_write, self.chosen(true)), page);
    }

    fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let keys: &gpui::Keystroke = &event.keystroke;
        let shift_only: bool = keys.modifiers == gpui::Modifiers::shift();
        if self.tick_key(keys.key.as_str(), shift_only) {
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if keys.modifiers.modified() {
            return;
        }
        match keys.key.as_str() {
            "up" => self.step(false),
            "down" => self.step(true),
            "enter" => self.choose(self.selected, window, cx),
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }

    /// Cmd+O on the page is its Open... entry.
    fn on_open(&mut self, _: &super::OpenWorkspace, window: &mut Window, cx: &mut Context<Self>) {
        self.ask(window, cx);
    }

    /// Cmd+Q on the page: a workspace window, if any, quits as its own Cmd+Q
    /// does (dirty tabs ask first); else the app quits.
    fn on_quit(_: &mut Self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        let workspace: Option<WindowHandle<ExplorerView>> = cx
            .windows()
            .into_iter()
            .find_map(|window: AnyWindowHandle| window.downcast::<ExplorerView>());
        match workspace {
            Some(handle) => {
                let _gone: gpui::Result<()> = handle.update(
                    cx,
                    |view: &mut ExplorerView,
                     window: &mut Window,
                     cx: &mut Context<ExplorerView>| {
                        view.quit(&Quit, window, cx);
                    },
                );
            }
            None => cx.quit(),
        }
    }

    fn row(
        &self,
        index: usize,
        theme: bb_ui::Theme,
        cx: &Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let (name, folder, open): (String, String, bool) =
            row_label(self.entries.get(index), self.home.as_deref());
        let (left, right): (gpui::Div, gpui::Div) = row_texts(name, folder, open, theme);
        div()
            .id(("start-entry", index))
            .debug_selector(move || format!("start-entry-{index}"))
            .px(px(10.0))
            .py(px(6.0))
            .rounded(px(4.0))
            .border_1()
            .border_color(color(if index == self.selected {
                theme.accent
            } else {
                theme.bg
            }))
            .hover(move |style: gpui::StyleRefinement| style.bg(color(theme.overlay)))
            .cursor_pointer()
            .flex()
            .justify_between()
            .gap(px(16.0))
            .on_click(cx.listener(
                move |page: &mut Self,
                      _: &ClickEvent,
                      window: &mut Window,
                      cx: &mut Context<Self>| {
                    page.choose(index, window, cx);
                },
            ))
            .child(left)
            .child(right)
    }
}

impl Render for StartPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme: bb_ui::Theme = crate::theme::resolved_theme(None, Some(window.appearance()));
        let rows: Vec<gpui::Stateful<gpui::Div>> = (0..self.entries.len())
            .map(|index: usize| self.row(index, theme, cx))
            .collect();
        let none: bool = self.entries.len() == 1;
        crate::launch::window_actions(crate::settings::root(cx))
            .id("start-page")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key))
            .on_action(cx.listener(Self::on_open))
            .on_action(cx.listener(Self::on_quit))
            .size_full()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(4.0))
            .bg(color(theme.bg))
            .text_color(color(theme.text))
            .child(header(theme))
            .child(self.ticks_row(theme, cx))
            .children(self.none_ticked_note(theme))
            .when(none, |page: gpui::Stateful<gpui::Div>| {
                page.child(empty_note(theme))
            })
            .child(rows_box(rows))
    }
}
