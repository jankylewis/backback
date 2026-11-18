//! A window's languages (BBEOD26): the start page's ticks or the saved
//! choice, what the folder holds, and the tools the window started. Every
//! tool start asks [`WindowLangs::start`]; nothing else decides.

use std::path::Path;

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
pub(crate) fn resolve(root: &Path, ticked: Option<LangSet>) -> Langs {
    if let Some(ticks) = ticked {
        crate::launch::recent::save_langs(root, ticks);
    }
    if let Some(ticks) = ticked.or_else(|| crate::launch::recent::langs_of(root)) {
        return Langs::new(ticks, detect_until(root, ticks, DETECT_BUDGET).found);
    }
    let found: Detection = detect(root, DETECT_BUDGET);
    if found.complete && !found.found.is_empty() {
        crate::launch::recent::save_langs(root, found.found);
    }
    Langs::new(default_ticks(found.found), found.found)
}

/// The status bar: the active languages and absent notes, then the C#
/// server's state while C# is active, then rust-analyzer's once started
/// (its install hint when missing, BBEOD27).
pub(crate) fn status_line(view: &ExplorerView) -> String {
    let mut parts: Vec<String> = vec![view.langs.langs.status()];
    if view.langs.langs.allows(Tool::Roslyn) {
        parts.push(view.intel.status_label());
    }
    let rust: String = view.rust.status.label();
    if view.langs.langs.allows(Tool::RustAnalyzer) && !rust.is_empty() {
        parts.push(rust);
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
