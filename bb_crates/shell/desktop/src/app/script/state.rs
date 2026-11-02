//! The `state` step: one line of what the window shows, for readback logs.

use gpui::{AsyncWindowContext, WeakEntity};

use crate::ExplorerView;

pub(super) fn state(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext) {
    let _gone: gpui::Result<()> = view.read_with(cx, |v: &ExplorerView, _: &gpui::App| {
        let tab: Option<&crate::editor::Tab> = v.tabs.active_tab();
        let hover: String = v
            .intel
            .hover
            .as_ref()
            .and_then(|h: &crate::intel::model::popups::HoverPopup| h.text.lines().next())
            .unwrap_or_default()
            .to_owned();
        let col: usize = tab.map_or(0, |t: &crate::editor::Tab| {
            let line: usize = t.document.cursor_line();
            let start: usize = t.document.line_start(line).map_or(0, bb_text::CharIdx::raw);
            t.document.cursor().raw().saturating_sub(start).saturating_add(1)
        });
        eprintln!(
            "bb_state status={:?} tab={:?} line={} col={col} read_only={} completion={:?} hover={hover:?} refs={:?} rename={:?} actions={:?} problems={} message={:?} link={:?}",
            v.intel.status_label(),
            tab.map(|t: &crate::editor::Tab| t.path.display().to_string()),
            tab.map_or(0, |t: &crate::editor::Tab| t.document.cursor_line().saturating_add(1)),
            tab.is_some_and(|t: &crate::editor::Tab| t.read_only),
            v.intel.completion.as_ref().map(|c: &crate::intel::model::popups::CompletionPopup| c.visible().len()),
            v.intel.references.as_ref().map(|r: &crate::intel::model::popups::PlacesPanel| r.places.len()),
            v.intel.rename_preview.as_ref().map(crate::intel::model::popups::RenamePreview::counts),
            v.intel.actions.as_ref().map(|a: &crate::intel::model::popups::ActionsPopup| a.items.len()),
            v.intel.problems().len(),
            v.status,
            v.intel.link.as_ref().map(|l: &crate::intel::model::popups::LinkTarget| (l.start.raw(), l.end.raw(), l.confirmed)),
        );
        if let Some(tab) = tab {
            let at: Vec<String> = v.intel.diagnostics_of(&tab.path).iter()
                .map(|d: &bb_intel::Diagnostic| format!("{}:{}", d.span.start.line.saturating_add(1), d.span.start.character.saturating_add(1)))
                .collect();
            eprintln!("bb_diag_state version={:?} squiggles={at:?}",
                v.intel.shown_version(&tab.path).map(bb_intel::DocVersion::raw));
        }
        let discovered: usize = v.tests.model.projects().values()
            .map(|project: &crate::tests_panel::model::ProjectTests| project.specs.len()).sum();
        let loading: usize = v.tests.model.projects().values()
            .filter(|project: &&crate::tests_panel::model::ProjectTests| project.loading).count();
        eprintln!("bb_tests_state projects={} discovered={discovered} loading={loading}",
            v.tests.model.projects().len());
    });
}
