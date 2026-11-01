//! The font settings as an app-wide value, loaded at start and saved on
//! every change (`settings` next to the recent list).

use std::path::PathBuf;

use bb_ui::Typography;
use gpui::{prelude::*, App, Global};

struct Fonts(Typography);

impl Global for Fonts {}

/// The font settings every surface draws with.
pub(crate) fn typography(cx: &App) -> Typography {
    cx.try_global::<Fonts>()
        .map_or_else(Typography::default, |fonts: &Fonts| fonts.0.clone())
}

/// The window root: the one font and size every surface inherits.
pub(crate) fn root(cx: &App) -> gpui::Div {
    let fonts: Typography = typography(cx);
    gpui::div()
        .font_family(fonts.family.clone())
        .text_size(gpui::px(fonts.size()))
}

/// Load the saved settings (defaults when none or unreadable).
pub(crate) fn install(cx: &mut App) {
    cx.set_global(Fonts(load()));
}

/// Change the settings, save them and redraw every window.
pub(crate) fn update(cx: &mut App, change: fn(&mut Typography)) {
    update_with(cx, change);
}

/// [`update`] with any change, e.g. a family picked in the Settings window.
pub(crate) fn update_with(cx: &mut App, change: impl FnOnce(&mut Typography)) {
    let mut fonts: Typography = typography(cx);
    change(&mut fonts);
    save(&fonts);
    apply(cx, fonts);
}

/// Use `fonts` everywhere now (no save): every window redraws with them.
pub(crate) fn apply(cx: &mut App, fonts: Typography) {
    cx.set_global(Fonts(fonts));
    cx.refresh_windows();
}

fn file() -> Option<PathBuf> {
    // Seam: unit tests never read or write the real settings file.
    if cfg!(test) {
        return None;
    }
    crate::launch::recent::state_dir().map(|dir: PathBuf| dir.join("settings"))
}

fn load() -> Typography {
    let Some(path): Option<PathBuf> = file() else {
        return Typography::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => Typography::decode(&text).unwrap_or_else(|error: bb_ui::SettingsError| {
            eprintln!("backback: {error}; using defaults");
            Typography::default()
        }),
        Err(_) => Typography::default(),
    }
}

fn save(fonts: &Typography) {
    let Some(path): Option<PathBuf> = file() else {
        return;
    };
    let tmp: PathBuf = path.with_extension("tmp");
    let written: std::io::Result<()> = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|(): ()| std::fs::write(&tmp, fonts.encode()))
        .and_then(|(): ()| std::fs::rename(&tmp, &path));
    if let Err(error) = written {
        eprintln!("backback: cannot save settings: {error}");
    }
}
