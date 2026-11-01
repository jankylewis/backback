//! Questions from the window. Each one flushes pending edits first, so the
//! server answers about the text the user sees.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bb_text::{CharIdx, Document, LspPoint};
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

use crate::editor::Tab;
use crate::intel::bb_worker::protocol::{Ask, Request, Ticket};
use crate::intel::model::popups::RenameInput;
use crate::intel::model::{LoadStatus, Question};
use crate::ExplorerView;

/// How long the mouse rests on a char before hover asks the server.
const HOVER_AFTER: Duration = Duration::from_millis(350);

/// Where a word containing `at` (or ending at `at`) starts.
pub(crate) fn word_start(document: &Document, at: CharIdx) -> CharIdx {
    let mut start: usize = at.raw();
    while let Some(before) = start.checked_sub(1) {
        match document.rope().get_char(before) {
            Some(ch) if ch.is_alphanumeric() || ch == '_' => start = before,
            Some(_) | None => break,
        }
    }
    CharIdx::new(start)
}

/// Ask about the cursor of the active tab. `anchor` is the start of the word
/// being completed. Returns `false` when nothing was sent.
pub(crate) fn ask_at_cursor(
    view: &mut ExplorerView,
    make: fn(LspPoint) -> Ask,
    anchor: Option<CharIdx>,
) -> bool {
    let Some((path, at, cursor)): Option<(PathBuf, LspPoint, CharIdx)> = cursor_point(view) else {
        return false;
    };
    ask(view, path, make(at), anchor.unwrap_or(cursor))
}

/// Send one question about `path`; flushes first.
pub(crate) fn ask(view: &mut ExplorerView, path: PathBuf, ask: Ask, anchor: CharIdx) -> bool {
    if view.link.is_none() || view.intel.status != LoadStatus::Ready {
        view.report(&format!("C# server: {}", view.intel.status.label()));
        return false;
    }
    super::flush(view);
    let revision: u64 = view
        .tabs
        .tab_mut(&path)
        .map_or(0, |tab: &mut Tab| tab.document.revision());
    let at: LspPoint = match &ask {
        Ask::Completion(p)
        | Ask::Hover(p)
        | Ask::Definition(p)
        | Ask::References(p)
        | Ask::Rename(p, _)
        | Ask::Actions(p, _) => *p,
    };
    let ticket: Ticket = view.intel.in_flight.ask(Question {
        kind: ask.kind(),
        path: path.clone(),
        revision,
        at,
        anchor,
    });
    super::send(
        view,
        Request::Ask {
            ticket,
            path,
            ask,
            sent: Instant::now(),
        },
    );
    true
}

/// The active tab's path, cursor as an LSP point, and cursor.
pub(crate) fn cursor_point(view: &ExplorerView) -> Option<(PathBuf, LspPoint, CharIdx)> {
    let tab: &Tab = view.tabs.active_tab()?;
    let cursor: CharIdx = tab.document.cursor();
    let at: LspPoint = tab.document.lsp_point(cursor).ok()?;
    Some((tab.path.clone(), at, cursor))
}

/// Ctrl+Space or `.`: complete the word at the cursor.
pub(crate) fn completion(view: &mut ExplorerView) -> bool {
    let anchor: Option<CharIdx> = view
        .tabs
        .active()
        .map(|doc: &Document| word_start(doc, doc.cursor()));
    ask_at_cursor(view, Ask::Completion, anchor)
}

/// Cmd+.: the code actions for the selection, or the cursor.
pub(crate) fn actions(view: &mut ExplorerView) -> bool {
    let Some(tab): Option<&Tab> = view.tabs.active_tab() else {
        return false;
    };
    let doc: &Document = &tab.document;
    let (from, to): (CharIdx, CharIdx) = doc.selection().unwrap_or((doc.cursor(), doc.cursor()));
    let (Ok(start), Ok(end)): (
        Result<LspPoint, bb_text::DocumentError>,
        Result<LspPoint, bb_text::DocumentError>,
    ) = (doc.lsp_point(from), doc.lsp_point(to)) else {
        return false;
    };
    let path: PathBuf = tab.path.clone();
    ask(view, path, Ask::Actions(start, end), from)
}

/// F2: open the new-name box with the current name.
pub(crate) fn rename_start(view: &mut ExplorerView) {
    let Some((path, at, cursor)): Option<(PathBuf, LspPoint, CharIdx)> = cursor_point(view) else {
        return;
    };
    let name: String = view
        .tabs
        .active()
        .map(|doc: &Document| {
            let start: CharIdx = word_start(doc, cursor);
            let end: usize = (cursor.raw()..doc.len_chars())
                .find(|i: &usize| {
                    !doc.rope()
                        .get_char(*i)
                        .is_some_and(|ch: char| ch.is_alphanumeric() || ch == '_')
                })
                .unwrap_or(doc.len_chars());
            doc.rope()
                .get_slice(start.raw()..end)
                .map(|part: bb_text::RopeSlice<'_>| part.to_string())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    view.intel.rename_input = Some(RenameInput {
        path,
        at,
        text: name,
    });
}

/// Enter in the new-name box: ask for the plan.
pub(crate) fn rename_submit(view: &mut ExplorerView) {
    let Some(input): Option<RenameInput> = view.intel.rename_input.take() else {
        return;
    };
    if input.text.is_empty() {
        return;
    }
    let anchor: CharIdx = view.tabs.active().map_or(CharIdx::new(0), Document::cursor);
    view.intel.rename_asked = Some(input.text.clone());
    ask(view, input.path, Ask::Rename(input.at, input.text), anchor);
}

/// The mouse moved over `at` in the active tab: ask for hover once it rests.
pub(crate) fn hover_move(
    view: &mut ExplorerView,
    at: Option<CharIdx>,
    position: (f32, f32),
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let path: Option<PathBuf> = view.tabs.active_tab().map(|tab: &Tab| tab.path.clone());
    let (Some(path), Some(at)): (Option<PathBuf>, Option<CharIdx>) = (path, at) else {
        view.intel.hover_target = None;
        return;
    };
    if let Some((old_path, old_at, _)) = &view.intel.hover_target {
        if *old_path == path && *old_at == at {
            return;
        }
    }
    let token: u64 = view
        .intel
        .hover_target
        .as_ref()
        .map_or(0, |target: &(PathBuf, CharIdx, u64)| target.2)
        .saturating_add(1);
    view.intel.hover = None;
    view.intel.hover_target = Some((path.clone(), at, token));
    view.intel.hover_at = position;
    let timer: gpui::Task<()> = cx.background_executor().timer(HOVER_AFTER);
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            timer.await;
            let _gone: gpui::Result<()> = view.update(
                cx,
                |view: &mut ExplorerView, _: &mut Context<ExplorerView>| {
                    hover_rested(view, path, at, token);
                },
            );
        },
    )
    .detach();
}

/// The hover timer fired: ask only if the mouse is still on the same char
/// and the server is ready.
fn hover_rested(view: &mut ExplorerView, path: PathBuf, at: CharIdx, token: u64) {
    let still: bool = view
        .intel
        .hover_target
        .as_ref()
        .is_some_and(|target: &(PathBuf, CharIdx, u64)| target.2 == token);
    let point: Option<LspPoint> = view
        .tabs
        .tab_mut(&path)
        .and_then(|tab: &mut Tab| tab.document.lsp_point(at).ok());
    let ready: bool = view.link.is_some() && view.intel.status == LoadStatus::Ready;
    if let (true, true, Some(point)) = (still, ready, point) {
        ask(view, path, Ask::Hover(point), at);
    }
}
