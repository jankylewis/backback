//! The `state` step: one line of what the window shows, for readback logs.

use gpui::{AsyncWindowContext, WeakEntity};

use crate::ExplorerView;

pub(super) fn state(view: &WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext) {
    let _gone: gpui::Result<()> = view.read_with(cx, |v: &ExplorerView, app: &gpui::App| {
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
            "bb_state status={:?} tab={:?} line={} col={col} read_only={} scroll_x={} completion={:?} hover={hover:?} refs={:?} rename={:?} actions={:?} problems={} message={:?} link={:?} root={:?}",
            v.intel.status_label(),
            tab.map(|t: &crate::editor::Tab| t.path.display().to_string()),
            tab.map_or(0, |t: &crate::editor::Tab| t.document.cursor_line().saturating_add(1)),
            tab.is_some_and(|t: &crate::editor::Tab| t.read_only),
            tab.map_or(0.0, |t: &crate::editor::Tab| t.scroll_x),
            v.intel.completion.as_ref().map(|c: &crate::intel::model::popups::CompletionPopup| c.visible().len()),
            v.intel.references.as_ref().map(|r: &crate::intel::model::popups::PlacesPanel| r.places.len()),
            v.intel.rename_preview.as_ref().map(crate::intel::model::popups::RenamePreview::counts),
            v.intel.actions.as_ref().map(|a: &crate::intel::model::popups::ActionsPopup| a.items.len()),
            v.intel.problems().len(),
            v.status,
            v.intel.link.as_ref().map(|l: &crate::intel::model::popups::LinkTarget| (l.start.raw(), l.end.raw(), l.confirmed)),
            v.root.display().to_string(),
        );
        panels(v, app);
        if let Some(tab) = tab {
            let at: Vec<String> = v.intel.diagnostics_of(&tab.path).iter()
                .map(|d: &bb_intel::Diagnostic| format!("{}:{}", d.span.start.line.saturating_add(1), d.span.start.character.saturating_add(1)))
                .collect();
            eprintln!("bb_diag_state version={:?} squiggles={at:?}",
                v.intel.shown_version(&tab.path).map(bb_intel::DocVersion::raw));
        }
        tests_line(v);
        tree_line(v);
        tabs_line(v);
        if let Some(tab) = tab {
            eprintln!("bb_debug_state breakpoints={:?} {}", v.debug.breakpoints.lines(&tab.path), crate::run::bb_debug::view::notes::describe(v));
        }
    });
}

/// The tab bar's line: names in order, `*` after a pinned one (BB-BUG-039).
fn tabs_line(v: &ExplorerView) {
    let names: Vec<String> = v
        .tabs
        .items()
        .iter()
        .map(|tab: &crate::editor::Tab| {
            let name: String = tab
                .path
                .file_name()
                .map_or_else(String::new, |n: &std::ffi::OsStr| {
                    n.to_string_lossy().into_owned()
                });
            if tab.pinned {
                format!("{name}*")
            } else {
                name
            }
        })
        .collect();
    eprintln!(
        "bb_tabs_state tabs={names:?} active={:?} asking={}",
        v.tabs.active_index().map(|i: usize| i.saturating_add(1)),
        v.tabs.pending_close()
    );
}

/// The Explorer's line: its selected row and how many rows show (BB-BUG-029).
fn tree_line(v: &ExplorerView) {
    let rows: Vec<(usize, &bb_explorer::Entry)> = v.tree.visible();
    let selected: Option<String> = rows
        .get(v.selected)
        .map(|(_, entry): &(usize, &bb_explorer::Entry)| entry.path().display().to_string());
    eprintln!("bb_tree_state selected={selected:?} rows={}", rows.len());
}

/// The Tests panel's line: what is listed and what runs.
fn tests_line(v: &ExplorerView) {
    let discovered: usize = v
        .tests
        .model
        .projects()
        .values()
        .map(|project: &crate::tests_panel::model::ProjectTests| project.specs.len())
        .sum();
    let loading: usize = v
        .tests
        .model
        .projects()
        .values()
        .filter(|project: &&crate::tests_panel::model::ProjectTests| project.loading)
        .count();
    let running: usize = v
        .tests
        .running_specs
        .values()
        .map(std::collections::BTreeSet::len)
        .sum();
    eprintln!("bb_tests_state projects={} discovered={discovered} loading={loading} busy={} running={running} message={:?}",
        v.tests.model.projects().len(), v.tests.busy.is_some(), v.tests.message);
}

/// The lines for the open panels: the places panel (BBEOD24) and the start
/// page (BBEOD23).
fn panels(v: &ExplorerView, app: &gpui::App) {
    places(v.intel.references.as_ref());
    if let Some(quick) = &v.quick_open {
        eprintln!("{}", quick.describe());
    }
    eprintln!("{}", crate::launch::start_state(app));
}

/// The places panel, when open, as shown: its title and each row's file and
/// 1-based line (BBEOD24: the callers of a symbol).
fn places(panel: Option<&crate::intel::model::popups::PlacesPanel>) {
    let Some(panel): Option<&crate::intel::model::popups::PlacesPanel> = panel else {
        return;
    };
    let rows: Vec<String> = panel
        .places
        .iter()
        .map(|place: &crate::intel::bb_worker::protocol::Place| {
            let file: String = match &place.location.uri {
                bb_intel::DocUri::File(path) => path
                    .as_path()
                    .file_name()
                    .map(|name: &std::ffi::OsStr| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                bb_intel::DocUri::Virtual(uri) => uri.clone(),
            };
            format!(
                "{file}:{}",
                place.location.span.start.line.saturating_add(1)
            )
        })
        .collect();
    eprintln!("bb_places_state title={:?} places={rows:?}", panel.title);
}
