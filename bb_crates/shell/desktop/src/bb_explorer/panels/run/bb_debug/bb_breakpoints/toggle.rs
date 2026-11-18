//! Toggling breakpoints: F9 on the cursor line, a click on a line number;
//! after an edit they follow their code.

use std::path::PathBuf;

use gpui::{Context, Window};

use crate::run::bb_debug::session;
use crate::ExplorerView;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ToggleBreakpoint;

/// F9: toggle the breakpoint on the active tab's cursor line.
pub(crate) fn on_toggle(
    view: &mut ExplorerView,
    _: &ToggleBreakpoint,
    _: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some((path, line)): Option<(PathBuf, usize)> = view
        .tabs
        .active_tab()
        .map(|tab: &crate::editor::Tab| (tab.path.clone(), tab.document.cursor_line()))
    else {
        return;
    };
    toggle_line(view, &path, line);
    cx.notify();
}

/// Toggle `line` of `path` and say so in the status bar; only a C# file
/// takes one (BBEOD26: Debug works only from C# targets).
pub(crate) fn toggle_line(view: &mut ExplorerView, path: &std::path::Path, line: usize) {
    if !view.langs.langs.can_debug_from(path) {
        view.status = Some(String::from(
            "Breakpoints are for C# files in a window with C# ticked",
        ));
        return;
    }
    let set: bool = view.debug.breakpoints.toggle(path, line);
    if let Some(live) = view.debug.live.as_ref() {
        let lines: Vec<usize> = view.debug.breakpoints.lines(path);
        live.send(session::Command::Breakpoints(path.to_path_buf(), lines));
    }
    let word: &str = if set { "set" } else { "removed" };
    view.status = Some(format!(
        "Breakpoint {word} at line {}",
        line.saturating_add(1)
    ));
}

/// After an edit of `path`: the line count went from `before` to `after`
/// and the cursor from line `from` to line `to`; breakpoints follow.
pub(crate) fn after_edit(
    view: &mut ExplorerView,
    path: &std::path::Path,
    (before, after): (usize, usize),
    (from, to): (usize, usize),
) {
    if before == after {
        return;
    }
    let delta: isize = isize::try_from(after)
        .unwrap_or(isize::MAX)
        .saturating_sub(isize::try_from(before).unwrap_or(isize::MAX));
    view.debug.breakpoints.shift(path, from.min(to), delta);
}
