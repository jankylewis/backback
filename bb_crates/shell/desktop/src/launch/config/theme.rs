use bb_ui::bb_theme::{Theme, ThemeKind};
use gpui::WindowAppearance;

pub(super) fn resolved_theme(
    override_kind: Option<ThemeKind>,
    appearance: Option<WindowAppearance>,
) -> Theme {
    let system_kind: Option<ThemeKind> = appearance.map(|value: WindowAppearance| match value {
        WindowAppearance::Light | WindowAppearance::VibrantLight => ThemeKind::Light,
        WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeKind::Dark,
    });
    Theme::for_kind(override_kind.or(system_kind))
}
