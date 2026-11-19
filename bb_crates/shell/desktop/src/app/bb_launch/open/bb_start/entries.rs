//! The start page's entries (BBEOD23): the recent workspaces, newest first,
//! then Open...; and how a row and a path are shown. A folder gone from
//! disk stays, greyed, with a remove button, as VS Code keeps it until
//! removed (BBEOD33; it was left out before).

use std::path::{Path, PathBuf};

use gpui::{div, prelude::*, px, ClickEvent, Context, Window};

use crate::color;

/// One entry of the page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StartEntry {
    /// A recent workspace folder.
    Recent(PathBuf),
    /// A recent workspace folder no longer on disk: it opens nothing.
    Gone(PathBuf),
    /// The macOS open panel.
    Open,
}

/// How a row looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Look {
    Recent,
    Gone,
    Open,
}

/// The page's entries: each recent workspace (gone or not), then Open....
pub(crate) fn entries(recent: Vec<PathBuf>) -> Vec<StartEntry> {
    recent
        .into_iter()
        .map(|path: PathBuf| {
            if path.exists() {
                StartEntry::Recent(path)
            } else {
                StartEntry::Gone(path)
            }
        })
        .chain([StartEntry::Open])
        .collect()
}

pub(crate) fn label(entry: &StartEntry) -> String {
    match entry {
        StartEntry::Recent(path) => format!("{} {}", name_of(path), path.display()),
        StartEntry::Gone(path) => {
            format!("{} {} (no longer exists)", name_of(path), path.display())
        }
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

/// A row's name, its folder (or the key of Open...) and its look.
pub(crate) fn row_label(entry: Option<&StartEntry>, home: Option<&Path>) -> (String, String, Look) {
    match entry {
        Some(StartEntry::Recent(path)) => (name_of(path), shown_path(path, home), Look::Recent),
        Some(StartEntry::Gone(path)) => (
            name_of(path),
            format!("no longer exists · {}", shown_path(path, home)),
            Look::Gone,
        ),
        Some(StartEntry::Open) | None => {
            (String::from("Open..."), String::from("Cmd+O"), Look::Open)
        }
    }
}

/// A row's two texts: the name (Open... in the accent colour, a gone folder
/// greyed), then the folder, cut with an ellipsis when it does not fit.
pub(super) fn row_texts(
    name: String,
    folder: String,
    look: Look,
    theme: bb_ui::Theme,
) -> (gpui::Div, gpui::Div) {
    let tint: bb_ui::Rgb = match look {
        Look::Recent => theme.text,
        Look::Gone => theme.text_dim,
        Look::Open => theme.accent,
    };
    let left: gpui::Div = div().flex_none().text_color(color(tint)).child(name);
    let right: gpui::Div = div()
        .min_w(px(0.0))
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_color(color(theme.text_dim))
        .child(folder);
    (left, right)
}

/// The page's heading: its title and what to do.
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
}

/// RECENT, directly above its rows (BBEOD33: it sat above LANGUAGES).
pub(super) fn recent_heading(theme: bb_ui::Theme) -> gpui::Div {
    div()
        .debug_selector(|| String::from("start-recent-heading"))
        .pt(px(16.0))
        .pb(px(4.0))
        .text_color(color(theme.text_dim))
        .child("RECENT")
}

/// The line under RECENT when no recent workspace is left.
pub(super) fn empty_note(theme: bb_ui::Theme) -> gpui::Div {
    div()
        .px(px(10.0))
        .py(px(6.0))
        .text_color(color(theme.text_dim))
        .child("No recent workspaces yet")
}

/// The rows under RECENT: ten recent rows and Open... fit the page; a
/// smaller window scrolls them, by wheel and by Up/Down (`scroll`).
pub(super) fn rows_box(
    rows: Vec<gpui::Stateful<gpui::Div>>,
    scroll: &gpui::ScrollHandle,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id("start-rows")
        .debug_selector(|| String::from("start-rows"))
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scroll()
        .track_scroll(scroll)
        .flex()
        .flex_col()
        .gap(px(4.0))
        .children(rows)
}

impl super::StartPage {
    pub(super) fn row(
        &self,
        index: usize,
        theme: bb_ui::Theme,
        cx: &Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let (name, folder, look): (String, String, Look) =
            row_label(self.entries.get(index), self.home.as_deref());
        let (left, right): (gpui::Div, gpui::Div) = row_texts(name, folder, look, theme);
        let border: bb_ui::Rgb = if index == self.selected {
            theme.accent
        } else {
            theme.bg
        };
        div()
            .id(("start-entry", index))
            .debug_selector(move || format!("start-entry-{index}"))
            .px(px(10.0))
            .py(px(6.0))
            .rounded(px(4.0))
            .border_1()
            .border_color(color(border))
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
            .when(look == Look::Gone, |row: gpui::Stateful<gpui::Div>| {
                row.debug_selector(move || format!("start-gone-{index}"))
                    .child(remove_button(index, theme, cx))
            })
    }

    /// Remove a gone folder's row from the page and the saved recent list;
    /// nothing on disk is touched. Any other row stays.
    pub(super) fn remove(&mut self, index: usize) {
        let Some(StartEntry::Gone(path)): Option<StartEntry> = self.entries.get(index).cloned()
        else {
            return;
        };
        crate::launch::recent::forget(&path);
        self.entries
            .retain(|entry: &StartEntry| *entry != StartEntry::Gone(path.clone()));
        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
        self.refresh_ticks();
    }
}

/// The × of a gone folder's row: a click removes the row only.
fn remove_button(
    index: usize,
    theme: bb_ui::Theme,
    cx: &Context<super::StartPage>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(("start-remove", index))
        .debug_selector(move || format!("start-remove-{index}"))
        .flex_none()
        .px(px(6.0))
        .text_color(color(theme.text_dim))
        .hover(move |style: gpui::StyleRefinement| style.text_color(color(theme.text)))
        .child("×")
        .on_click(cx.listener(
            move |page: &mut super::StartPage,
                  _: &ClickEvent,
                  _: &mut Window,
                  cx: &mut Context<super::StartPage>| {
                page.remove(index);
                cx.stop_propagation();
                cx.notify();
            },
        ))
}
