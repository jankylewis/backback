//! A window's languages (BBEOD26): the start page's ticks or the saved
//! choice, what the folder holds, and the tools the window started. Every
//! tool start asks [`WindowLangs::start`]; nothing else decides.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bb_lang::{
    default_ticks, detect, detect_until, Detection, LangSet, Langs, Tool, DETECT_BUDGET,
};

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
}

impl WindowLangs {
    pub(crate) fn new(langs: Langs) -> Self {
        Self {
            langs,
            started: Vec::new(),
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

/// The ticks a window opening `root` uses: `ticked` from the start page
/// (saved for next time), else the saved choice, else the languages the
/// folder holds (saved once the walk saw the whole folder; C# alone for a
/// folder that holds none, unsaved). With ticks
/// known, the walk stops once each ticked language is seen.
/// A launch's walk, when [`walk_early`] started it, is used instead of a
/// walk on the UI thread (BB-PRF-007).
pub(crate) fn resolve(root: &Path, ticked: Option<LangSet>) -> Langs {
    if let Some(ticks) = ticked {
        crate::launch::recent::save_langs(root, ticks);
    }
    let ticks: Option<LangSet> = ticked.or_else(|| crate::launch::recent::langs_of(root));
    let found: Detection = early_walk(root, ticks).unwrap_or_else(|| walk(root, ticks));
    if let Some(ticks) = ticks {
        return Langs::new(ticks, found.found);
    }
    if found.complete && !found.found.is_empty() {
        crate::launch::recent::save_langs(root, found.found);
    }
    Langs::new(default_ticks(found.found), found.found)
}

/// The walk [`resolve`] needs: up to the known ticks, else the whole folder.
fn walk(root: &Path, ticks: Option<LangSet>) -> Detection {
    #[cfg(test)]
    record_walk(root);
    match ticks {
        Some(ticks) => detect_until(root, ticks, DETECT_BUDGET),
        None => detect(root, DETECT_BUDGET),
    }
}

/// A walk started before the app ran: its folder, its ticks, the thread.
struct Early {
    root: PathBuf,
    ticks: Option<LangSet>,
    walk: std::thread::JoinHandle<Detection>,
}

static EARLY: Mutex<Option<Early>> = Mutex::new(None);

/// A command-line launch: start the walk of `root` on its own thread now, so
/// it runs while the app and the window start, not on the UI thread before
/// the first frame (BB-PRF-007). The saved choice is read here, on the
/// calling thread, as [`resolve`] reads it.
pub(crate) fn walk_early(root: &Path, ticked: Option<LangSet>) {
    let ticks: Option<LangSet> = ticked.or_else(|| crate::launch::recent::langs_of(root));
    let owned: PathBuf = root.to_path_buf();
    let spawned: std::io::Result<std::thread::JoinHandle<Detection>> = std::thread::Builder::new()
        .name(String::from("bb-detect"))
        .spawn(move || walk(&owned, ticks));
    let Ok(walk): std::io::Result<std::thread::JoinHandle<Detection>> = spawned else {
        return;
    };
    if let Ok(mut slot) = EARLY.lock() {
        *slot = Some(Early {
            root: root.to_path_buf(),
            ticks,
            walk,
        });
    }
}

/// The early walk's result when it was for `root` with `ticks`.
fn early_walk(root: &Path, ticks: Option<LangSet>) -> Option<Detection> {
    let early: Early = EARLY.lock().ok()?.take()?;
    if early.root != root || early.ticks != ticks {
        return None;
    }
    early.walk.join().ok()
}

/// The status bar: the active languages and absent notes, then the C#
/// server's state while C# is active, then rust-analyzer's and pyright's
/// once started (each one's install hint when missing, BBEOD27/28).
pub(crate) fn status_line(view: &ExplorerView) -> String {
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
    if dirty {
        view.report(&"Languages: save or close the unsaved tabs first, then reopen");
        cx.notify();
        return;
    }
    let replace: gpui::AnyWindowHandle = window.window_handle();
    let current: LangSet = view.langs.langs.ticked();
    super::bb_start::reopen(&view.root, current, replace, cx);
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
