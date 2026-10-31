//! Light and dark: follow macOS, or a View menu choice.

use crate::ExplorerView;
use bb_ui::bb_theme::{Theme, ThemeKind};
use gpui::{App, Window, WindowAppearance};

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleTheme;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct FollowAppearance;

pub(crate) fn resolved_theme(
    override_kind: Option<ThemeKind>,
    appearance: Option<WindowAppearance>,
) -> Theme {
    let system_kind: Option<ThemeKind> = appearance.map(|value: WindowAppearance| match value {
        WindowAppearance::Light | WindowAppearance::VibrantLight => ThemeKind::Light,
        WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeKind::Dark,
    });
    Theme::for_kind(override_kind.or(system_kind))
}

pub(crate) fn install_appearance_observer(
    window: &Window,
    view: &gpui::Entity<ExplorerView>,
    cx: &mut App,
) {
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

/// A theme colour as GPUI paints it.
pub(crate) fn color(value: bb_ui::Rgb) -> gpui::Rgba {
    let packed: u32 = (u32::from(value.r) << 16) | (u32::from(value.g) << 8) | u32::from(value.b);
    gpui::rgb(packed)
}
