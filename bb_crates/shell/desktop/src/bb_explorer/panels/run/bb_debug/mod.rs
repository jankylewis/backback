//! The debugger (BBEOD17), keys as VS Code and Rider. Breakpoints: F9 on the
//! cursor line, or a click on a line number (the cursor stays); the number
//! is drawn on a red tag. The session (`session/`): F5 start or continue,
//! Shift+F5 stop, F10 / F11 / Shift+F11 step; `panel` shows the call stack
//! and variables while paused.

mod bb_breakpoints;
pub(crate) mod session;
#[cfg(test)]
mod tests;
pub(crate) mod view;

use std::path::PathBuf;

use gpui::{Context, Window};

pub(crate) use bb_breakpoints::Breakpoints;

use crate::ExplorerView;

/// The debugger's window state.
#[derive(Debug, Default)]
pub(crate) struct DebugState {
    /// Every breakpoint.
    pub(crate) breakpoints: Breakpoints,
    /// The running session, if any.
    pub(crate) live: Option<session::control::Live>,
}

macro_rules! debug_action {
    ($($name:ident),*) => {$(
        #[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
        #[action(namespace = backback)]
        pub(crate) struct $name;
    )*};
}
debug_action!(StartOrContinue, StopDebugging, StepOver, StepInto, StepOut);

/// The debug keys (registered on the window root).
pub(crate) fn actions<E: gpui::InteractiveElement>(area: E, cx: &mut Context<ExplorerView>) -> E {
    area.on_action(cx.listener(on_toggle))
        .on_action(cx.listener(
            |v: &mut ExplorerView,
             _: &StartOrContinue,
             w: &mut Window,
             cx: &mut Context<ExplorerView>| {
                session::control::start_or_continue(v, w, cx);
                cx.notify();
            },
        ))
        .on_action(cx.listener(
            |v: &mut ExplorerView,
             _: &StopDebugging,
             _: &mut Window,
             _: &mut Context<ExplorerView>| {
                session::control::stop(v);
            },
        ))
        .on_action(cx.listener(
            |v: &mut ExplorerView, _: &StepOver, _: &mut Window, _: &mut Context<ExplorerView>| {
                session::control::step(v, session::StepKind::Over);
            },
        ))
        .on_action(cx.listener(
            |v: &mut ExplorerView, _: &StepInto, _: &mut Window, _: &mut Context<ExplorerView>| {
                session::control::step(v, session::StepKind::Into);
            },
        ))
        .on_action(cx.listener(
            |v: &mut ExplorerView, _: &StepOut, _: &mut Window, _: &mut Context<ExplorerView>| {
                session::control::step(v, session::StepKind::Out);
            },
        ))
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

/// The 0-based line of `path` the debugger is paused on, if any.
pub(crate) fn stopped_line(view: &ExplorerView, path: &std::path::Path) -> Option<usize> {
    view::notes::stopped_at(view)
        .filter(|(file, _): &(PathBuf, usize)| file == path)
        .map(|(_, line): (PathBuf, usize)| line)
}
