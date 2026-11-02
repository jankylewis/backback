//! The link state: which word, asked or confirmed, and how it is drawn.

use std::path::{Path, PathBuf};

use bb_syntax::HighlightKind;
use bb_text::{CharIdx, Document, LspPoint};
use gpui::{prelude::*, Context, ModifiersChangedEvent, Window};

use crate::intel::bb_worker::protocol::{Ask, Place};
use crate::intel::model::popups::LinkTarget;
use crate::intel::model::{LoadStatus, Pending};
use crate::ExplorerView;

/// The identifier (letters, digits, `_`) that holds the char at `at`.
pub(crate) fn word_at(document: &Document, at: CharIdx) -> Option<(CharIdx, CharIdx)> {
    if !is_word(document, at.raw()) {
        return None;
    }
    let mut start: usize = at.raw();
    while let Some(before) = start
        .checked_sub(1)
        .filter(|i: &usize| is_word(document, *i))
    {
        start = before;
    }
    let mut end: usize = at.raw().saturating_add(1);
    while is_word(document, end) {
        end = end.saturating_add(1);
    }
    Some((CharIdx::new(start), CharIdx::new(end)))
}

fn is_word(document: &Document, at: usize) -> bool {
    document
        .rope()
        .get_char(at)
        .is_some_and(|ch: char| ch.is_alphanumeric() || ch == '_')
}

/// True when the char at `at` is inside a string literal or a comment, as
/// the editor colours it: those words are never links (Rider, VS do the same).
fn in_text(tab: &crate::editor::Tab, at: CharIdx) -> bool {
    let rope: &bb_text::Rope = tab.document.rope();
    let Some(byte): Option<usize> = rope.try_char_to_byte(at.raw()).ok() else {
        return false;
    };
    tab.highlighter
        .highlight_spans(tab.live.tree(), rope, byte..byte.saturating_add(1))
        .iter()
        .rev()
        .find(|span: &&bb_syntax::HighlightSpan| span.start <= byte && byte < span.end)
        .is_some_and(|span: &bb_syntax::HighlightSpan| {
            matches!(span.kind, HighlightKind::String | HighlightKind::Comment)
        })
}

/// The mouse is over `at` of the active tab; `cmd` is the Cmd key's state.
pub(crate) fn link_move(view: &mut ExplorerView, at: Option<CharIdx>, cmd: bool) {
    let found: Option<(PathBuf, (CharIdx, CharIdx))> = cmd
        .then(|| view.tabs.active_tab())
        .flatten()
        .and_then(|tab: &crate::editor::Tab| {
            word_at(&tab.document, at?)
                .filter(|range: &(CharIdx, CharIdx)| !in_text(tab, range.0))
                .map(|range: (CharIdx, CharIdx)| (tab.path.clone(), range))
        });
    let Some((path, (start, end))): Option<(PathBuf, (CharIdx, CharIdx))> = found else {
        view.intel.link = None;
        return;
    };
    let same: bool = view.intel.link.as_ref().is_some_and(|link: &LinkTarget| {
        link.path == path && link.start == start && link.end == end
    });
    if same {
        return;
    }
    view.intel.link = Some(LinkTarget {
        path: path.clone(),
        start,
        end,
        confirmed: false,
    });
    let ready: bool = view.link.is_some() && view.intel.status == LoadStatus::Ready;
    let point: Option<LspPoint> = view
        .tabs
        .active()
        .and_then(|doc: &Document| doc.lsp_point(start).ok());
    if let (true, Some(point)) = (ready, point) {
        crate::intel::glue::ask(view, path, Ask::Link(point), start);
    }
}

/// Cmd went down or up while the mouse rests on a char.
pub(crate) fn on_modifiers(
    view: &mut ExplorerView,
    event: &ModifiersChangedEvent,
    _: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let resting: Option<CharIdx> = view
        .intel
        .hover_target
        .as_ref()
        .map(|(_, at, _): &(PathBuf, CharIdx, u64)| *at);
    link_move(view, resting, event.modifiers.platform);
    cx.notify();
}

/// The server's answer to a link question: confirm the link still shown,
/// or drop it when the symbol has no definition.
pub(crate) fn link_answer(view: &mut ExplorerView, pending: &Pending, places: &[Place]) {
    let Some(link): Option<&mut LinkTarget> = view.intel.link.as_mut() else {
        return;
    };
    if link.path != pending.path || link.start != pending.anchor {
        return;
    }
    if places.is_empty() {
        view.intel.link = None;
    } else {
        link.confirmed = true;
    }
}

/// The editor area's link handling: Cmd press/release, and the hand cursor
/// while a confirmed link is under the mouse.
pub(crate) fn on_editor(
    area: gpui::Stateful<gpui::Div>,
    view: &ExplorerView,
    cx: &mut Context<ExplorerView>,
) -> gpui::Stateful<gpui::Div> {
    let hand: bool = view
        .intel
        .link
        .as_ref()
        .is_some_and(|link: &LinkTarget| link.confirmed);
    area.on_modifiers_changed(cx.listener(on_modifiers))
        .when(hand, gpui::Styled::cursor_pointer)
}

/// The confirmed link on `path`, as the char range to draw.
pub(crate) fn shown(view: &ExplorerView, path: &Path) -> Option<(CharIdx, CharIdx)> {
    view.intel
        .link
        .as_ref()
        .filter(|link: &&LinkTarget| link.confirmed && link.path == path)
        .map(|link: &LinkTarget| (link.start, link.end))
}
