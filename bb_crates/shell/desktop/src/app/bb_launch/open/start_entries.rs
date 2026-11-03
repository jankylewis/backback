//! The start page's entries (BBEOD23): the recent workspaces still on
//! disk, newest first, then Open...; and how a path is shown.

use std::path::{Path, PathBuf};

use gpui::{div, prelude::*, px};

use crate::color;

/// One entry of the page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StartEntry {
    /// A recent workspace folder.
    Recent(PathBuf),
    /// The macOS open panel.
    Open,
}

/// The page's entries: the recent workspaces still on disk, then Open....
pub(crate) fn entries(recent: Vec<PathBuf>) -> Vec<StartEntry> {
    recent
        .into_iter()
        .filter(|path: &PathBuf| path.exists())
        .map(StartEntry::Recent)
        .chain([StartEntry::Open])
        .collect()
}

pub(crate) fn label(entry: &StartEntry) -> String {
    match entry {
        StartEntry::Recent(path) => format!("{} {}", name_of(path), path.display()),
        StartEntry::Open => String::from("Open..."),
    }
}

pub(crate) fn name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name: &std::ffi::OsStr| name.to_string_lossy().into_owned(),
    )
}

/// `path` with the home folder shown as `~`, as Finder and Rider show it.
pub(crate) fn shown_path(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|home: &Path| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => String::from("~"),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// A row's name, its folder (or the key of Open...) and whether it is Open....
pub(super) fn row_label(entry: Option<&StartEntry>, home: Option<&Path>) -> (String, String, bool) {
    match entry {
        Some(StartEntry::Recent(path)) => (name_of(path), shown_path(path, home), false),
        Some(StartEntry::Open) | None => (String::from("Open..."), String::from("Cmd+O"), true),
    }
}

/// A row's two texts: the name (Open... in the accent colour), then the
/// folder, cut with an ellipsis when it does not fit.
pub(super) fn row_texts(
    name: String,
    folder: String,
    open: bool,
    theme: bb_ui::Theme,
) -> (gpui::Div, gpui::Div) {
    let left: gpui::Div = div()
        .flex_none()
        .text_color(color(if open { theme.accent } else { theme.text }))
        .child(name);
    let right: gpui::Div = div()
        .min_w(px(0.0))
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_color(color(theme.text_dim))
        .child(folder);
    (left, right)
}

/// The page's heading: its title, what to do, and RECENT.
pub(super) fn header(theme: bb_ui::Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child("Welcome to backback")
        .child(
            div()
                .text_color(color(theme.text_dim))
                .child("Open a recent workspace, or a folder, solution or file."),
        )
        .child(
            div()
                .pt(px(16.0))
                .pb(px(4.0))
                .text_color(color(theme.text_dim))
                .child("RECENT"),
        )
}

/// The line under RECENT when no recent workspace is left.
pub(super) fn empty_note(theme: bb_ui::Theme) -> gpui::Div {
    div()
        .px(px(10.0))
        .py(px(6.0))
        .text_color(color(theme.text_dim))
        .child("No recent workspaces yet")
}
