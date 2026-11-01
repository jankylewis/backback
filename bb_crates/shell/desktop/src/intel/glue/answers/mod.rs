//! What each accepted answer does to the window.

mod bb_rename;

pub(crate) use bb_rename::apply_rename;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bb_intel::{CompletionEntry, DocUri, HoverInfo};
use bb_text::{CharIdx, LspPoint};
use gpui::{Context, ScrollStrategy, Window};

use crate::editor::Tab;
use crate::intel::bb_worker::protocol::{Answer, Place, Stages};
use crate::intel::model::popups::{
    ActionsPopup, CompletionPopup, HoverPopup, Pick, PlacesPanel, RenamePreview,
};
use crate::intel::model::{Jump, Pending, Undrawn};
use crate::ExplorerView;

pub(super) fn apply(
    view: &mut ExplorerView,
    pending: &Pending,
    answer: Answer,
    stages: Stages,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    match answer {
        Answer::Completion(items) => completion(view, pending, items, stages),
        Answer::Hover(info) => hover(view, pending, info),
        Answer::Definition(places) => match places.into_iter().next() {
            Some(place) => jump(view, &place, window, cx),
            None => view.report(&"No definition found"),
        },
        Answer::References(places) => {
            view.intel.references = Some(PlacesPanel {
                title: format!("{} references", places.len()),
                places,
                pick: Pick::default(),
            });
        }
        Answer::Rename(changes) => {
            let revisions: BTreeMap<PathBuf, u64> = view
                .tabs
                .items()
                .iter()
                .map(|tab: &Tab| (tab.path.clone(), tab.document.revision()))
                .collect();
            view.intel.rename_preview = Some(RenamePreview {
                name: view.intel.rename_asked.take().unwrap_or_default(),
                changes,
                revisions,
            });
        }
        Answer::Actions(items) if items.is_empty() => view.report(&"No code actions here"),
        Answer::Actions(items) => {
            view.intel.actions = Some(ActionsPopup {
                items,
                pick: Pick::default(),
            });
        }
        Answer::Failed(reason) => view.report(&format!("C# server: {reason}")),
    }
}

fn completion(
    view: &mut ExplorerView,
    pending: &Pending,
    items: Vec<CompletionEntry>,
    stages: Stages,
) {
    if items.is_empty() {
        view.intel.completion = None;
        return;
    }
    let mut popup: CompletionPopup =
        CompletionPopup::new(pending.path.clone(), pending.anchor, items);
    if let Some(typed) = view
        .tabs
        .active()
        .and_then(|doc: &bb_text::Document| super::typed_since(doc, pending.anchor))
    {
        popup.set_filter(&typed);
    }
    view.intel.completion = Some(popup);
    view.intel.undrawn = Some(Undrawn {
        stages,
        applied: Instant::now(),
        frame: view.intel.frame,
    });
}

/// Show hover text only while the mouse still rests on the char it was
/// asked for; the mouse may have moved on while the server worked.
fn hover(view: &mut ExplorerView, pending: &Pending, info: Option<HoverInfo>) {
    let resting: bool =
        view.intel
            .hover_target
            .as_ref()
            .is_some_and(|(path, at, _): &(PathBuf, CharIdx, u64)| {
                *path == pending.path && *at == pending.anchor
            });
    if !resting {
        return;
    }
    let line: u32 = info
        .as_ref()
        .and_then(|i: &HoverInfo| i.span)
        .map_or(0, |s: bb_intel::Span| s.start.line);
    view.intel.hover =
        info.filter(|i: &HoverInfo| !i.text.trim().is_empty())
            .map(|i: HoverInfo| HoverPopup {
                text: i.text,
                line,
                x: view.intel.hover_at.0,
                y: view.intel.hover_at.1,
            });
}

/// Go to a place: switch to its tab, or open it and move there once it has
/// loaded. A file outside the workspace (decompiled metadata) opens read-only.
pub(crate) fn jump(
    view: &mut ExplorerView,
    place: &Place,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let DocUri::File(path): &DocUri = &place.location.uri else {
        view.report(&"That definition has no source file");
        return;
    };
    let path: PathBuf = path
        .as_path()
        .canonicalize()
        .unwrap_or_else(|_: std::io::Error| path.as_path().to_path_buf());
    let at: LspPoint = place.location.span.start;
    let read_only: bool = !path.starts_with(&view.root);
    if view.tabs.tab_mut(&path).is_some() {
        if let Some(index) = view
            .tabs
            .items()
            .iter()
            .position(|tab: &Tab| tab.path == path)
        {
            view.tabs.switch(index);
        }
        move_cursor(view, &path, at);
        window.focus(&view.editor_focus, cx);
        return;
    }
    view.intel.jump = Some(Jump {
        path: path.clone(),
        at,
        read_only,
    });
    view.open_file(&path, window, cx);
}

/// A tab finished opening: run the jump that waited for it.
pub(crate) fn opened(view: &mut ExplorerView, path: &Path) {
    let Some(jump): Option<Jump> = view.intel.jump.take_if(|j: &mut Jump| j.path == path) else {
        return;
    };
    if let Some(tab) = view
        .tabs
        .tab_mut(path)
        .filter(|_: &&mut Tab| jump.read_only)
    {
        tab.set_read_only();
    }
    move_cursor(view, path, jump.at);
}

/// Put the cursor of `path`'s tab at `at` and scroll it into view.
pub(crate) fn move_cursor(view: &mut ExplorerView, path: &Path, at: LspPoint) {
    let Some(tab): Option<&mut Tab> = view.tabs.tab_mut(path) else {
        return;
    };
    let Ok(target): Result<CharIdx, bb_text::DocumentError> = tab.document.char_of_lsp(at) else {
        return;
    };
    if tab.document.set_cursor(target).is_ok() {
        let line: usize = usize::try_from(at.line).unwrap_or(0);
        view.editor_scroll
            .scroll_to_item(line, ScrollStrategy::Center);
    }
}
