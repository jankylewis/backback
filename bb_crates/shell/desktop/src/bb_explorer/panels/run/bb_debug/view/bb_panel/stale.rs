//! A paused file that changed since the session's build (BB-BUG-072/073):
//! the running code is the build's, so a name typed after it (`int x = 1;` in
//! an unsaved tab) has no value. The status bar and the VARIABLES header
//! say so, and a hover the debugger cannot answer there says so too,
//! instead of the server's type for code that is not running.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use bb_ui::bb_theme::Theme;
use gpui::{prelude::*, AnyElement};

use crate::intel::view::one_line;
use crate::{color, ExplorerView};

/// What the notice says.
pub(crate) const NOTICE: &str = "file changed since the debug build — restart to rebuild";

/// The rule: unsaved edits, or saved after the session began (its build
/// starts then).
pub(crate) fn changed_since(dirty: bool, saved: Option<SystemTime>, started: SystemTime) -> bool {
    dirty || saved.is_some_and(|saved: SystemTime| saved > started)
}

/// Whether `path` changed since the running session began.
pub(crate) fn file_changed(view: &ExplorerView, path: &Path) -> bool {
    let Some(started): Option<SystemTime> = view.debug.started else {
        return false;
    };
    let dirty: bool = view
        .tabs
        .items()
        .iter()
        .any(|tab: &crate::editor::Tab| tab.path == path && tab.document.is_dirty());
    let saved: Option<SystemTime> = std::fs::metadata(path)
        .and_then(|meta: std::fs::Metadata| meta.modified())
        .ok();
    changed_since(dirty, saved, started)
}

/// The paused file, when it changed since the session began.
pub(crate) fn paused_file_changed(view: &ExplorerView) -> Option<PathBuf> {
    let (path, _): (PathBuf, usize) = super::super::notes::stopped_at(view)?;
    file_changed(view, &path).then_some(path)
}

/// The status at a stop in a changed file: `<file>: <notice>`.
pub(crate) fn said_at_a_stop(view: &ExplorerView) -> Option<String> {
    let path: PathBuf = paused_file_changed(view)?;
    let name: std::borrow::Cow<'_, str> = path.file_name()?.to_string_lossy();
    Some(format!("{name}: {NOTICE}"))
}

/// The notice row atop VARIABLES, when the paused file changed.
pub(crate) fn notice_row(view: &ExplorerView, theme: Theme) -> Option<AnyElement> {
    paused_file_changed(view)?;
    Some(
        one_line(20.0)
            .id("vars-stale")
            .debug_selector(|| "vars-stale".to_owned())
            .px_2()
            .text_color(color(theme.warn))
            .child(NOTICE)
            .into_any_element(),
    )
}

#[cfg(test)]
mod tests {
    use super::changed_since;
    use std::time::{Duration, SystemTime};

    use gpui::{TestAppContext, VisualTestContext};
    use tokio::sync::mpsc::UnboundedReceiver;

    use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
    use crate::run::bb_debug::session::{Command, Note, Var};
    use crate::ExplorerView;

    #[test]
    fn a_file_is_stale_when_unsaved_or_saved_after_the_build_began() {
        let started: SystemTime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000);
        let before: SystemTime = started - Duration::from_secs(5);
        let after: SystemTime = started + Duration::from_secs(5);
        assert!(
            !changed_since(false, Some(before), started),
            "built as saved"
        );
        assert!(changed_since(true, Some(before), started), "unsaved edits");
        assert!(changed_since(false, Some(after), started), "saved after");
        assert!(!changed_since(false, None, started), "no file time");
    }

    /// BB-BUG-072: paused in a file edited after the debug build (`int x = 1;`
    /// typed, unsaved), the status and VARIABLES say so, and a name the
    /// debugger cannot evaluate says so instead of the server's type.
    #[gpui::test]
    fn a_file_changed_since_the_debug_build_says_so(cx: &mut TestAppContext) {
        let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
        let _commands: UnboundedReceiver<Command> =
            super::super::tests::paused(&rig, vcx, Vec::new());
        let file: std::path::PathBuf = rig.file.clone();
        let at: bb_text::CharIdx = bb_text::CharIdx::new(4);
        vcx.run_until_parked();
        assert!(vcx.debug_bounds("vars-stale").is_none(), "built as shown");
        rig.view.update(
            vcx,
            |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
                v.debug.started = Some(std::time::SystemTime::now());
                v.intel.hover_target = Some((file.clone(), at, 9));
                let tab: &mut crate::editor::Tab = v.tabs.tab_mut(&file).unwrap();
                tab.document.insert("int x = 1;").unwrap();
            },
        );
        let frames: Vec<crate::run::bb_debug::session::Frame> =
            vec![super::super::tests::frame(&rig)];
        let reason: String = String::from("breakpoint");
        let stop: Note = Note::Stopped {
            reason,
            exception: None,
            frames,
            vars: Vec::new(),
        };
        super::super::tests::note(&rig, vcx, stop);
        let notice: &str = super::NOTICE;
        assert!(
            vcx.debug_bounds("vars-stale").is_some(),
            "VARIABLES says so"
        );
        let value: Result<Var, String> = Err(String::from("CS0103: x"));
        super::super::tests::note(&rig, vcx, Note::Evaluated { token: 9, value });
        let (status, hover): (Option<String>, Option<String>) = said(&rig, vcx);
        assert!(
            status.is_some_and(|s: String| s.ends_with(notice)),
            "the status says so"
        );
        assert_eq!(
            hover,
            Some(format!("no value: {notice}")),
            "never the server's type"
        );
    }

    /// The status and the hover's text.
    fn said(rig: &Rig, vcx: &mut VisualTestContext) -> (Option<String>, Option<String>) {
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            let hover: Option<&crate::intel::model::popups::HoverPopup> = v.intel.hover.as_ref();
            (
                v.status.clone(),
                hover.map(|h: &crate::intel::model::popups::HoverPopup| h.text.clone()),
            )
        })
    }
}
