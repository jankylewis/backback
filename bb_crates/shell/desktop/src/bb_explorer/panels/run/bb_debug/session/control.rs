//! The session as the window sees it (keys as VS Code and Rider): F5 starts
//! the startup project, or continues when paused; Shift+F5 stops; F10, F11
//! and Shift+F11 step. Notes from the session thread update the window: a
//! stop opens its line and fills the Debug panel.

use std::path::PathBuf;

use bb_dotnet::{ProjectInfo, Solution, SolutionPath};
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{start, Command, Frame, Note, Start, StepKind, Var};
use crate::ExplorerView;

/// A running session.
#[derive(Debug)]
pub(crate) struct Live {
    commands: UnboundedSender<Command>,
    /// The startup project's name.
    pub(crate) label: String,
    pub(crate) phase: Phase,
    /// The program's output, newest last.
    pub(crate) output: String,
    /// The session's notes when no task pumps them (unit tests poll).
    pending: Option<UnboundedReceiver<Note>>,
    /// The adapter can move the next statement (BBEOD05).
    pub(crate) can_set_next: bool,
}

/// Where the session is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Building,
    Running,
    Stopped {
        frames: Vec<Frame>,
        /// The picked call-stack row.
        selected: usize,
        vars: Vec<Var>,
        /// The exception paused on, as `Type: message`.
        exception: Option<String>,
    },
}

impl Live {
    /// Queue a command; false when the session thread is gone.
    pub(crate) fn send(&self, command: Command) -> bool {
        self.commands.send(command).is_ok()
    }
}

/// F5: start, or continue when paused.
pub(crate) fn start_or_continue(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if let Some(live) = view.debug.live.as_ref() {
        if matches!(live.phase, Phase::Stopped { .. }) && live.send(Command::Continue) {
            return;
        }
        view.status = Some(format!("{} is already running", live.label));
        return;
    }
    if view.tabs.no_write() {
        view.report(&"Debug refused: --no-write keeps the solution unchanged (a debug build writes bin/ and obj/)");
        return;
    }
    let Some((target, label)): Option<(PathBuf, String)> = startup_project(view) else {
        view.report(&"Debug: the solution has no console (Exe) project to start");
        return;
    };
    let (commands, notes): (UnboundedSender<Command>, UnboundedReceiver<Note>) = start(Start {
        target,
        breakpoints: view.debug.breakpoints.all(),
        exceptions: view.debug.exceptions.filters(),
    });
    view.debug.live = Some(Live {
        commands,
        label,
        phase: Phase::Building,
        output: String::new(),
        pending: Some(notes),
        can_set_next: false,
    });
    // Seam: gpui's test scheduler refuses wake-ups from the session thread;
    // unit tests drain the notes with `poll` (as the terminal does).
    if !cfg!(test) {
        if let Some(notes) = view
            .debug
            .live
            .as_mut()
            .and_then(|l: &mut Live| l.pending.take())
        {
            pump(notes, window, cx);
        }
    }
}

/// A paused session with no thread behind it: window tests of the panel.
/// Returns the commands the window sends.
#[cfg(test)]
pub(crate) fn fake_paused(
    view: &mut ExplorerView,
    frames: Vec<Frame>,
    exception: Option<String>,
) -> UnboundedReceiver<Command> {
    let (commands, sent): (UnboundedSender<Command>, UnboundedReceiver<Command>) =
        tokio::sync::mpsc::unbounded_channel();
    view.debug.live = Some(Live {
        commands,
        label: String::from("Fake"),
        phase: Phase::Stopped {
            frames,
            selected: 0,
            vars: Vec::new(),
            exception,
        },
        output: String::new(),
        pending: None,
        can_set_next: false,
    });
    sent
}

/// Apply the notes that have arrived (unit tests, see the seam above).
#[cfg(test)]
pub(crate) fn poll(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let Some(mut notes): Option<UnboundedReceiver<Note>> = view
        .debug
        .live
        .as_mut()
        .and_then(|l: &mut Live| l.pending.take())
    else {
        return;
    };
    while let Ok(note) = notes.try_recv() {
        crate::run::bb_debug::view::notes::apply(view, note, window, cx);
        cx.notify();
    }
    if let Some(live) = view.debug.live.as_mut() {
        live.pending = Some(notes);
    }
}

/// F10, F11, Shift+F11 while paused.
pub(crate) fn step(view: &mut ExplorerView, kind: StepKind) {
    if let Some(live) = view.debug.live.as_ref() {
        if matches!(live.phase, Phase::Stopped { .. }) {
            live.send(Command::Step(kind));
        }
    }
}

/// Ctrl+Shift+F10 while paused: the active tab's cursor line runs next (Set
/// Next Statement, BBEOD05), on an adapter that advertises it.
pub(crate) fn set_next(view: &mut ExplorerView) {
    let Some((path, line)): Option<(PathBuf, usize)> = view
        .tabs
        .active_tab()
        .map(|tab: &crate::editor::Tab| (tab.path.clone(), tab.document.cursor_line()))
    else {
        return;
    };
    if can_set_next(view) {
        if let Some(live) = view.debug.live.as_ref() {
            live.send(Command::SetNext(path, line));
        }
    }
}

/// Paused on an adapter that can Set Next: the action exists only then, so
/// its key does nothing and its menu item greys otherwise.
pub(crate) fn can_set_next(view: &ExplorerView) -> bool {
    view.debug
        .live
        .as_ref()
        .is_some_and(|live: &Live| live.can_set_next && matches!(live.phase, Phase::Stopped { .. }))
}

/// Shift+F5.
pub(crate) fn stop(view: &mut ExplorerView) {
    if let Some(live) = view.debug.live.as_ref() {
        live.send(Command::Stop);
    }
}

/// The console project to debug: the one holding the active file, else the
/// solution's first (test projects never).
fn startup_project(view: &ExplorerView) -> Option<(PathBuf, String)> {
    let solution: PathBuf = crate::intel::bb_worker::find_solution(&view.root)?;
    let listing: Solution = SolutionPath::new(solution)
        .and_then(|path: SolutionPath| Solution::load(&path))
        .ok()?;
    let apps: Vec<ProjectInfo> = listing
        .projects
        .iter()
        .filter_map(|project: &bb_dotnet::ProjectPath| ProjectInfo::load(project).ok())
        .filter(|info: &ProjectInfo| info.is_exe && !info.is_test)
        .collect();
    let active: Option<PathBuf> = view
        .tabs
        .active_tab()
        .map(|tab: &crate::editor::Tab| tab.path.clone());
    let pick: &ProjectInfo = apps
        .iter()
        .find(|info: &&ProjectInfo| holds(info, active.as_deref()))
        .or_else(|| apps.first())?;
    Some((pick.path.as_path().to_path_buf(), pick.name.clone()))
}

fn holds(info: &ProjectInfo, file: Option<&std::path::Path>) -> bool {
    let dir: Option<&std::path::Path> = info.path.as_path().parent();
    matches!((file, dir), (Some(file), Some(dir)) if file.starts_with(dir))
}

fn pump(mut notes: UnboundedReceiver<Note>, window: &mut Window, cx: &mut Context<ExplorerView>) {
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            while let Some(note) = notes.recv().await {
                let delivered: gpui::Result<()> = view.update_in(
                    cx,
                    |v: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                        crate::run::bb_debug::view::notes::apply(v, note, window, cx);
                        cx.notify();
                    },
                );
                if delivered.is_err() {
                    break;
                }
            }
        },
    )
    .detach();
}
