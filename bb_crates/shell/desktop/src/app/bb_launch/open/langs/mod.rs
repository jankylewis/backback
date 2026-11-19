//! A window's languages (BBEOD26): the start page's ticks or the saved
//! choice, what the folder holds, and the tools the window started. Every
//! tool start asks [`WindowLangs::start`]; nothing else decides.

#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Mutex;

use bb_lang::{LangSet, Langs, Tool};

mod bb_early;
#[cfg(test)]
pub(crate) use bb_early::{hold_walks, release_walks};
pub(crate) use bb_early::{resolve, walk_early, when_found, Pending, Resolved};

use crate::ExplorerView;

/// File > Languages...: reopen this window with other ticks.
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct ChangeLanguages;

/// The languages of one workspace window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WindowLangs {
    pub(crate) langs: Langs,
    /// The tools this window started, oldest first.
    pub(crate) started: Vec<Tool>,
    /// The launch's walk has not ended yet: nothing is active, no tool
    /// starts, and the status bar says so (BB-PRF-009).
    pub(crate) detecting: bool,
}

impl WindowLangs {
    pub(crate) fn new(langs: Langs) -> Self {
        Self {
            langs,
            started: Vec::new(),
            detecting: false,
        }
    }

    /// A window shown before its languages are known (BB-PRF-009).
    pub(crate) fn detecting() -> Self {
        let none: LangSet = LangSet::EMPTY;
        Self {
            langs: Langs::new(none, none),
            started: Vec::new(),
            detecting: true,
        }
    }

    /// Whether `tool` may start; when it may, it is recorded as started.
    #[must_use]
    pub(crate) fn start(&mut self, tool: Tool) -> bool {
        let allowed: bool = self.langs.allows(tool);
        if allowed && !self.started.contains(&tool) {
            self.started.push(tool);
        }
        allowed
    }
}

/// The status bar: the active languages and absent notes, then the C#
/// server's state while C# is active, then rust-analyzer's and pyright's
/// once started (each one's install hint when missing, BBEOD27/28).
pub(crate) fn status_line(view: &ExplorerView) -> String {
    if view.langs.detecting {
        return String::from(DETECTING);
    }
    let mut parts: Vec<String> = vec![view.langs.langs.status()];
    if view.langs.langs.allows(Tool::Roslyn) {
        parts.push(view.intel.status_label());
    }
    if view.langs.langs.allows(Tool::RustAnalyzer) {
        parts.push(view.rust.status.label(bb_intel::Server::RustAnalyzer));
    }
    if view.langs.langs.allows(Tool::Pyright) {
        parts.push(view.python.status.label(bb_intel::Server::Pyright));
    }
    parts.retain(|part: &String| !part.is_empty());
    parts.join(" · ")
}

/// File > Languages...: the start page for this workspace; its Open
/// reopens the window with the new ticks. Unsaved tabs refuse first.
pub(crate) fn change(
    view: &mut ExplorerView,
    _: &ChangeLanguages,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<ExplorerView>,
) {
    let dirty: bool = view
        .tabs
        .items()
        .iter()
        .any(|tab: &crate::editor::Tab| tab.document.is_dirty());
    if view.langs.detecting {
        view.report(&"Languages: still detecting languages; try again in a moment");
        cx.notify();
        return;
    }
    if dirty {
        view.report(&"Languages: save or close the unsaved tabs first, then reopen");
        cx.notify();
        return;
    }
    let replace: gpui::AnyWindowHandle = window.window_handle();
    let current: LangSet = view.langs.langs.ticked();
    super::bb_start::reopen(&view.root, current, replace, cx);
}

/// The status bar while the launch's walk runs (BB-PRF-009).
pub(crate) const DETECTING: &str = "Detecting languages…";

/// What Debug says before the window's languages are known.
pub(crate) const DEBUG_DETECTING: &str = "Debug: detecting languages; try again in a moment";

/// F5 while the launch's walk runs: Debug is not shown yet, so it says why.
pub(crate) fn debug_while_detecting(
    view: &mut ExplorerView,
    _: &crate::run::bb_debug::StartOrContinue,
    _: &mut gpui::Window,
    cx: &mut gpui::Context<ExplorerView>,
) {
    view.report(&DEBUG_DETECTING);
    cx.notify();
}

/// What Debug says in a window without C# active.
pub(crate) const NO_DEBUG: &str =
    "Debug: C# is not active in this window (there is no Rust or Python debugger)";

/// Whether Debug may start: C# active (BBEOD26, checked first) and saves
/// allowed (a debug build writes bin/ and obj/); else the status says why.
pub(crate) fn may_debug(view: &mut ExplorerView) -> bool {
    if !view.langs.langs.debug_visible() {
        view.report(&NO_DEBUG);
        return false;
    }
    if view.tabs.no_write() {
        view.report(&"Debug refused: --no-write keeps the solution unchanged (a debug build writes bin/ and obj/)");
        return false;
    }
    true
}

/// Tests: which thread walked which folder (BB-PRF-007).
#[cfg(test)]
static WALKS: Mutex<Vec<(PathBuf, std::thread::ThreadId)>> = Mutex::new(Vec::new());

#[cfg(test)]
fn record_walk(root: &Path) {
    if let Ok(mut walks) = WALKS.lock() {
        walks.push((root.to_path_buf(), std::thread::current().id()));
    }
}

/// Tests: the threads that walked `root`, oldest first.
#[cfg(test)]
pub(crate) fn walks_of(root: &Path) -> Vec<std::thread::ThreadId> {
    let Ok(walks): Result<std::sync::MutexGuard<'_, Vec<(PathBuf, std::thread::ThreadId)>>, _> =
        WALKS.lock()
    else {
        return Vec::new();
    };
    walks
        .iter()
        .filter(|(path, _): &&(PathBuf, std::thread::ThreadId)| path == root)
        .map(|(_, id): &(PathBuf, std::thread::ThreadId)| *id)
        .collect()
}
