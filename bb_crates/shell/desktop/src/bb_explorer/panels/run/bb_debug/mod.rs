//! The debugger (BBEOD17). Part 1: breakpoints. F9 toggles one on the
//! cursor line; a click on a line number toggles that line (the cursor
//! stays); the number is drawn on a red tag.

mod bb_breakpoints;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use gpui::{Context, Window};

pub(crate) use bb_breakpoints::Breakpoints;

use crate::ExplorerView;

/// The debugger's window state.
#[derive(Debug, Default)]
pub(crate) struct DebugState {
    /// Every breakpoint.
    pub(crate) breakpoints: Breakpoints,
}

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

/// Toggle `line` of `path` and say so in the status bar.
pub(crate) fn toggle_line(view: &mut ExplorerView, path: &std::path::Path, line: usize) {
    let set: bool = view.debug.breakpoints.toggle(path, line);
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
