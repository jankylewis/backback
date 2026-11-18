//! The debugger (BBEOD17), keys as VS Code and Rider. Breakpoints: F9 on the
//! cursor line, or a click on a line number (the cursor stays); the number
//! is drawn on a red tag. Exception breakpoints (BBEOD19): the Debug menu's
//! two ticks, All and User-Unhandled, kept per workspace. The session
//! (`session/`): F5 start or continue, Shift+F5 stop, F10 / F11 / Shift+F11
//! step; `panel` shows the call stack and variables while paused.

mod bb_breakpoints;
pub(crate) mod session;
#[cfg(test)]
mod tests;
pub(crate) mod view;

use std::path::PathBuf;

use gpui::{Context, Window};

pub(crate) use bb_breakpoints::{
    after_edit, on_toggle, toggle_line, Breakpoints, ExceptionBreaks, ExceptionKind,
    ToggleBreakpoint,
};

use crate::ExplorerView;

/// The debugger's window state.
#[derive(Debug, Default)]
pub(crate) struct DebugState {
    /// Every breakpoint.
    pub(crate) breakpoints: Breakpoints,
    /// Which exceptions pause the program.
    pub(crate) exceptions: ExceptionBreaks,
    /// The running session, if any.
    pub(crate) live: Option<session::control::Live>,
    /// The test the session debugs (its row shows ■, BB-FEAT-081).
    pub(crate) test: Option<bb_testkit::Fqn>,
    /// Call Stack and Variables scroll across (BB-FEAT-085).
    pub(crate) stack_scroll: gpui::ScrollHandle,
    pub(crate) vars_scroll: gpui::ScrollHandle,
    /// The Variables tree's opened rows and loaded children (BB-FEAT-087).
    pub(crate) tree: view::VarTree,
    /// When the running session began: its build starts then (BB-BUG-072).
    pub(crate) started: Option<std::time::SystemTime>,
}

impl DebugState {
    /// The state for the workspace at `root`: no breakpoints yet, and the
    /// exception ticks it saved last time.
    pub(crate) fn load(root: &std::path::Path) -> Self {
        Self {
            breakpoints: Breakpoints::default(),
            exceptions: ExceptionBreaks::load(root),
            live: None,
            test: None,
            stack_scroll: gpui::ScrollHandle::new(),
            vars_scroll: gpui::ScrollHandle::new(),
            tree: view::VarTree::default(),
            started: None,
        }
    }
}

macro_rules! debug_action {
    ($($name:ident),*) => {$(
        #[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
        #[action(namespace = backback)]
        pub(crate) struct $name;
    )*};
}
debug_action!(
    StartOrContinue,
    StopDebugging,
    StepOver,
    StepInto,
    StepOut,
    BreakOnAllExceptions,
    BreakOnUserUnhandledExceptions,
    SetNextStatement
);

/// The debug keys (registered on the window root). Set Next Statement only
/// while it can run (`set_next`), so its menu item greys otherwise.
pub(crate) fn actions<E: gpui::InteractiveElement>(
    area: E,
    set_next: bool,
    cx: &mut Context<ExplorerView>,
) -> E {
    let keys: E = area
        .on_action(cx.listener(on_toggle))
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
        ));
    exception_actions(set_next_action(keys, set_next, cx), cx)
}

/// The exception ticks' actions (the Debug menu).
fn exception_actions<E: gpui::InteractiveElement>(area: E, cx: &mut Context<ExplorerView>) -> E {
    area.on_action(cx.listener(
        |v: &mut ExplorerView,
         _: &BreakOnAllExceptions,
         _: &mut Window,
         cx: &mut Context<ExplorerView>| {
            toggle_exceptions(v, ExceptionKind::All);
            cx.notify();
        },
    ))
    .on_action(cx.listener(
        |v: &mut ExplorerView,
         _: &BreakOnUserUnhandledExceptions,
         _: &mut Window,
         cx: &mut Context<ExplorerView>| {
            toggle_exceptions(v, ExceptionKind::UserUnhandled);
            cx.notify();
        },
    ))
}

/// Flip one exception tick: saved for the workspace, sent to a live
/// session, said in the status bar.
pub(crate) fn toggle_exceptions(view: &mut ExplorerView, kind: ExceptionKind) {
    let on: bool = view.debug.exceptions.toggle(kind);
    view.debug.exceptions.save(&view.root);
    if let Some(live) = view.debug.live.as_ref() {
        live.send(session::Command::Exceptions(
            view.debug.exceptions.filters(),
        ));
    }
    let state: &str = if on { "on" } else { "off" };
    view.status = Some(format!("Break on {}: {state}", kind.label().to_lowercase()));
}

/// Ctrl+Shift+F10, only while Set Next can run (BBEOD05): without the
/// handler the key does nothing and the menu item greys.
fn set_next_action<E: gpui::InteractiveElement>(
    keys: E,
    set_next: bool,
    cx: &mut Context<ExplorerView>,
) -> E {
    if !set_next {
        return keys;
    }
    keys.on_action(cx.listener(
        |v: &mut ExplorerView,
         _: &SetNextStatement,
         _: &mut Window,
         _: &mut Context<ExplorerView>| {
            session::control::set_next(v);
        },
    ))
}

/// The Debug menu: the keys of VS Code and Rider, then the exception ticks.
pub(crate) fn debug_menu() -> gpui::Menu {
    use gpui::MenuItem;
    gpui::Menu::new("Debug").items([
        MenuItem::action("Start / Continue", StartOrContinue),
        MenuItem::action("Stop Debugging", StopDebugging),
        MenuItem::separator(),
        MenuItem::action("Step Over", StepOver),
        MenuItem::action("Step Into", StepInto),
        MenuItem::action("Step Out", StepOut),
        MenuItem::action("Set Next Statement", SetNextStatement),
        MenuItem::separator(),
        MenuItem::action("Toggle Breakpoint", ToggleBreakpoint),
        MenuItem::action("Break on All Exceptions", BreakOnAllExceptions),
        MenuItem::action(
            "Break on User-Unhandled Exceptions",
            BreakOnUserUnhandledExceptions,
        ),
    ])
}

/// The 0-based line of `path` the debugger is paused on, if any.
pub(crate) fn stopped_line(view: &ExplorerView, path: &std::path::Path) -> Option<usize> {
    view::notes::stopped_at(view)
        .filter(|(file, _): &(PathBuf, usize)| file == path)
        .map(|(_, line): (PathBuf, usize)| line)
}
