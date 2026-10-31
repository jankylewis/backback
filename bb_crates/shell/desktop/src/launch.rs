use crate::editor::Tabs;
use crate::{ExplorerView, MoveDown, MoveLeft, MoveRight, MoveUp, Open};
use bb_explorer::Explorer;
use bb_ui::bb_theme::{Theme, ThemeKind};
use gpui::{
    px, size, App, AppContext, Bounds, FocusHandle, KeyBinding, Menu, MenuItem, Window,
    WindowAppearance, WindowBounds, WindowOptions,
};
use std::{path::PathBuf, time::Instant};

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleTheme;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct FollowAppearance;

pub(crate) fn launch() {
    let start: Instant = Instant::now();
    let path: PathBuf = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let tree: Explorer = match Explorer::open(&path) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
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
                let view: gpui::Entity<ExplorerView> = new_view(tree, window.appearance(), cx);
                let focus: FocusHandle = view.read(cx).focus.clone();
                window.focus(&focus, cx);
                install_close_guard(window, &view, cx);
                install_appearance_observer(window, &view, cx);
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

fn menus() -> Vec<Menu> {
    vec![
        Menu::new("Backback"),
        Menu::new("View").items([
            MenuItem::action("Toggle Light/Dark Theme", ToggleTheme),
            MenuItem::action("Follow macOS Appearance", FollowAppearance),
        ]),
    ]
}

fn new_view(
    tree: Explorer,
    appearance: WindowAppearance,
    cx: &mut App,
) -> gpui::Entity<ExplorerView> {
    cx.new(|cx: &mut gpui::Context<ExplorerView>| ExplorerView {
        tree,
        selected: 0,
        focus: cx.focus_handle(),
        editor_focus: cx.focus_handle(),
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
    })
}

fn resolved_theme(override_kind: Option<ThemeKind>, appearance: Option<WindowAppearance>) -> Theme {
    let system_kind: Option<ThemeKind> = appearance.map(|value: WindowAppearance| match value {
        WindowAppearance::Light | WindowAppearance::VibrantLight => ThemeKind::Light,
        WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeKind::Dark,
    });
    Theme::for_kind(override_kind.or(system_kind))
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
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_preference_is_dark_and_system_light_is_live_source() {
        assert_eq!(resolved_theme(None, None).kind, ThemeKind::Dark);
        assert_eq!(
            resolved_theme(None, Some(WindowAppearance::Light)).kind,
            ThemeKind::Light
        );
        assert_eq!(
            resolved_theme(None, Some(WindowAppearance::Dark)).kind,
            ThemeKind::Dark
        );
        assert_eq!(
            resolved_theme(Some(ThemeKind::Dark), Some(WindowAppearance::Light)).kind,
            ThemeKind::Dark
        );
    }

    #[test]
    fn view_is_a_separate_mac_menu_after_application_menu() {
        let configured: Vec<Menu> = menus();
        assert_eq!(configured[0].name.as_ref(), "Backback");
        assert_eq!(configured[1].name.as_ref(), "View");
        assert_eq!(configured[1].items.len(), 2);
    }
}
