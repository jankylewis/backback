//! The start page (BBEOD23), as Rider's Welcome screen: shown on a Finder or
//! Dock launch with no argument and by File > Start Page.... It lists the
//! recent workspaces, newest first, then Open.... A click, or Up/Down and
//! Enter, opens an entry as a command-line argument would, then the page
//! closes.

use std::path::PathBuf;

use gpui::{
    div, prelude::*, px, size, AnyWindowHandle, App, Bounds, ClickEvent, Context, FocusHandle,
    KeyDownEvent, Subscription, TitlebarOptions, Window, WindowBounds, WindowHandle, WindowOptions,
};

use super::start_entries::{empty_note, entries, header, label, row_label, row_texts, StartEntry};
use crate::editor::Quit;
use crate::{color, ExplorerView};

pub(crate) struct StartPage {
    entries: Vec<StartEntry>,
    selected: usize,
    no_write: bool,
    home: Option<PathBuf>,
    focus: FocusHandle,
    _appearance: Subscription,
}

/// Show the page: bring an open one to the front, else open one.
pub(crate) fn show(no_write: bool, cx: &mut App) {
    let shown: Option<WindowHandle<StartPage>> = cx
        .windows()
        .into_iter()
        .find_map(|window: AnyWindowHandle| window.downcast::<StartPage>());
    if let Some(page) = shown {
        let _gone: gpui::Result<()> = page.update(
            cx,
            |_: &mut StartPage, window: &mut Window, _: &mut Context<StartPage>| {
                window.activate_window();
            },
        );
        return;
    }
    if let Some(page) = open(crate::launch::recent::list(), no_write, cx) {
        super::start_script::drive(page, cx);
    }
}

/// A page for `recent` in a new window, focused.
pub(crate) fn open(
    recent: Vec<PathBuf>,
    no_write: bool,
    cx: &mut App,
) -> Option<WindowHandle<StartPage>> {
    let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(640.0), px(460.0)), cx);
    let opened: gpui::Result<WindowHandle<StartPage>> = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Welcome to backback".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
        |window: &mut Window, cx: &mut App| {
            let page: gpui::Entity<StartPage> =
                cx.new(|cx: &mut Context<StartPage>| StartPage::new(recent, no_write, window, cx));
            let focus: FocusHandle = page.read(cx).focus.clone();
            window.focus(&focus, cx);
            page
        },
    );
    match opened {
        Ok(page) => Some(page),
        Err(error) => {
            eprintln!("backback: cannot open the start page: {error}");
            None
        }
    }
}

impl StartPage {
    /// The page for `recent`, newest first; folders gone from disk are left out.
    pub(crate) fn new(
        recent: Vec<PathBuf>,
        no_write: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            entries: entries(recent),
            selected: 0,
            no_write,
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
    }

    /// Open entry `index`: a workspace in its own window (the page closes), or
    /// the open panel (the page closes once a pick opens).
    fn choose(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = index;
        match self.entries.get(index).cloned() {
            Some(StartEntry::Recent(path)) => {
                let start: crate::args::Start = crate::args::start_for(path, self.no_write);
                if crate::launch::open_workspace(&start, cx) {
                    window.remove_window();
                }
            }
            Some(StartEntry::Open) => super::ask(cx, self.no_write, Some(window.window_handle())),
            None => {}
        }
        cx.notify();
    }

    fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.modifiers.modified() {
            return;
        }
        match event.keystroke.key.as_str() {
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
        super::ask(cx, self.no_write, Some(window.window_handle()));
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
            .when(none, |page: gpui::Stateful<gpui::Div>| {
                page.child(empty_note(theme))
            })
            .children(rows)
    }
}
