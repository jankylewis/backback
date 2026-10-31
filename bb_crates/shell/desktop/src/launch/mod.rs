mod bb_menu;
use bb_menu::menus;
#[path = "config/bb_path.rs"]
mod path;
use path::split_argument;
#[path = "config/theme.rs"]
mod theme;
use crate::editor::{Quit, Tabs};
use crate::harness::Harness;
use crate::{ExplorerView, HarnessVisibility, MoveDown, MoveLeft, MoveRight, MoveUp, Open};
use bb_explorer::Explorer;
use bb_ui::bb_theme::{Theme, ThemeKind};
use gpui::{
    px, size, App, AppContext, Bounds, FocusHandle, KeyBinding, Window, WindowAppearance,
    WindowBounds, WindowOptions,
};
use std::time::Duration;
use std::{path::PathBuf, time::Instant};
use theme::resolved_theme;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleHarness;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleTheme;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct FollowAppearance;

pub(crate) fn launch() {
    let start: Instant = Instant::now();
    let (tree, harness, file): (Explorer, Harness, Option<PathBuf>) = initial_state();
    gpui_platform::application().run(move |cx: &mut App| {
        bind_keys(cx);
        cx.set_menus(menus());
        let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);
        let result: gpui::Result<gpui::WindowHandle<ExplorerView>> = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window: &mut Window, cx: &mut App| {
                let view: gpui::Entity<ExplorerView> =
                    new_view(tree, harness, window.appearance(), cx);
                let focus: FocusHandle = view.read(cx).focus.clone();
                window.focus(&focus, cx);
                install_close_guard(window, &view, cx);
                install_appearance_observer(window, &view, cx);
                install_harness_poll(window, &view, cx);
                #[cfg(debug_assertions)]
                crate::harness::demo::start(&view, window, cx);
                if let Some(file) = &file {
                    view.update(
                        cx,
                        |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                            state.open_file(file, window, cx);
                        },
                    );
                }
                window.on_next_frame(move |_: &mut Window, _: &mut App| {
                    eprintln!("first_frame_ms={}", start.elapsed().as_millis());
                });
                view
            },
        );
        if let Err(error) = result {
            eprintln!("{error}");
            cx.quit();
        }
        cx.activate(true);
    });
}

fn initial_state() -> (Explorer, Harness, Option<PathBuf>) {
    let argument: PathBuf = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let (root, file): (PathBuf, Option<PathBuf>) = split_argument(argument);
    let tree: Explorer = match Explorer::open(&root) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let harness: Harness = match Harness::load(&root) {
        Ok(harness) => harness,
        Err(error) => {
            eprintln!("{error}");
            Harness::empty()
        }
    };
    (tree, harness, file)
}

pub(crate) fn new_view(
    tree: Explorer,
    harness: Harness,
    appearance: WindowAppearance,
    cx: &mut App,
) -> gpui::Entity<ExplorerView> {
    cx.new(|cx: &mut gpui::Context<ExplorerView>| ExplorerView {
        tree,
        selected: 0,
        focus: cx.focus_handle(),
        editor_focus: cx.focus_handle(),
        harness_focus: cx.focus_handle(),
        harness,
        show_harness: HarnessVisibility::Visible,
        tabs: Tabs::new(),
        keypresses: 0,
        latency_before: None,
        frame_collector: None,
        edit_ns: 0,
        reparse_ns: 0,
        closing_window: false,
        panel_width: 300.0,
        resizing: false,
        theme: resolved_theme(None, Some(appearance)),
        theme_override: None,
        appearance_subscription: None,
        status: None,
        explorer_scroll: gpui::UniformListScrollHandle::new(),
        quit_after_close: false,
    })
}

fn install_appearance_observer(window: &Window, view: &gpui::Entity<ExplorerView>, cx: &mut App) {
    let weak: gpui::WeakEntity<ExplorerView> = view.downgrade();
    let subscription: gpui::Subscription =
        window.observe_window_appearance(move |window: &mut Window, cx: &mut App| {
            weak.update(
                cx,
                |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                    state.theme = resolved_theme(state.theme_override, Some(window.appearance()));
                    cx.notify();
                },
            )
            .ok();
        });
    view.update(
        cx,
        |state: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            state.appearance_subscription = Some(subscription);
        },
    );
}

impl ExplorerView {
    pub(crate) fn toggle_harness(
        &mut self,
        _: &ToggleHarness,
        _: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.show_harness = match self.show_harness {
            HarnessVisibility::Hidden => HarnessVisibility::Visible,
            HarnessVisibility::Visible => HarnessVisibility::Hidden,
        };
        cx.notify();
    }
    pub(crate) fn toggle_theme(
        &mut self,
        _: &ToggleTheme,
        _: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let next: ThemeKind = match self.theme.kind {
            ThemeKind::Dark => ThemeKind::Light,
            ThemeKind::Light => ThemeKind::Dark,
        };
        self.theme_override = Some(next);
        self.theme = Theme::for_kind(Some(next));
        cx.set_window_appearance(Some(match next {
            ThemeKind::Light => WindowAppearance::Light,
            ThemeKind::Dark => WindowAppearance::Dark,
        }));
        cx.notify();
    }

    pub(crate) fn follow_appearance(
        &mut self,
        _: &FollowAppearance,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.theme_override = None;
        cx.set_window_appearance(None);
        self.theme = resolved_theme(None, Some(window.appearance()));
        cx.notify();
    }
}

fn install_harness_poll(window: &mut Window, view: &gpui::Entity<ExplorerView>, cx: &mut App) {
    let weak: gpui::WeakEntity<ExplorerView> = view.downgrade();
    view.update(
        cx,
        |_: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            cx.spawn_in(
                window,
                async move |_: gpui::WeakEntity<ExplorerView>,
                            cx: &mut gpui::AsyncWindowContext| {
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(100))
                            .await;
                        if weak.update_in(cx, |state: &mut ExplorerView, _: &mut Window,
                    cx: &mut gpui::Context<ExplorerView>| {
                    if state.harness.poll() { cx.notify(); }
                }).is_err() { break; }
                    }
                },
            )
            .detach();
        },
    );
}

fn install_close_guard(window: &Window, view: &gpui::Entity<ExplorerView>, cx: &App) {
    let close_view: gpui::Entity<ExplorerView> = view.clone();
    window.on_window_should_close(cx, move |_: &mut Window, cx: &mut App| {
        close_view.update(
            cx,
            |state: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                state.handle_window_close(cx)
            },
        )
    });
}

fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, Some("Explorer")),
        KeyBinding::new("down", MoveDown, Some("Explorer")),
        KeyBinding::new("left", MoveLeft, Some("Explorer")),
        KeyBinding::new("right", MoveRight, Some("Explorer")),
        KeyBinding::new("enter", Open, Some("Explorer")),
        KeyBinding::new("cmd-q", Quit, None),
    ]);
}

#[cfg(test)]
#[path = "tests.rs"]
mod bb_tests;
