//! Keys for code intelligence. [`key`] runs before the editor's own keys and
//! returns `true` when it used the key; [`after_edit`] runs after a typed
//! edit to keep the completion list in step with the text.

use bb_text::{CharIdx, Document};
use gpui::{Context, KeyDownEvent, Keystroke, Window};

use super::asks;
use crate::intel::bb_worker::protocol::Ask;
use crate::intel::model::popups::CompletionPopup;
use crate::ExplorerView;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct RestartServer;

impl ExplorerView {
    pub(crate) fn restart_server(
        &mut self,
        _: &RestartServer,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        super::restart(self, window, cx);
    }
}

/// Handle a key meant for a popup or an intel command.
pub(crate) fn key(view: &mut ExplorerView, event: &KeyDownEvent) -> bool {
    let stroke: &Keystroke = &event.keystroke;
    view.intel.hover = None;
    if view.intel.rename_input.is_some() {
        rename_box(view, stroke);
        return true;
    }
    if view.intel.rename_preview.is_some() && preview_key(view, stroke.key.as_str()) {
        return true;
    }
    if list_key(view, stroke) {
        return true;
    }
    command(view, stroke)
}

fn command(view: &mut ExplorerView, stroke: &Keystroke) -> bool {
    let m: &gpui::Modifiers = &stroke.modifiers;
    match stroke.key.as_str() {
        "f12" => {
            let ask: fn(bb_text::LspPoint) -> Ask = if m.shift {
                Ask::References
            } else {
                Ask::Definition
            };
            asks::ask_at_cursor(view, ask, None);
            true
        }
        "f2" => {
            asks::rename_start(view);
            true
        }
        "space" if m.control => {
            asks::completion(view);
            true
        }
        "." if m.platform => {
            asks::actions(view);
            true
        }
        "m" if m.platform && m.shift => {
            view.intel.problems_open = !view.intel.problems_open;
            true
        }
        "escape" => view.intel.dismiss() || close_panels(view),
        _ => false,
    }
}

fn close_panels(view: &mut ExplorerView) -> bool {
    let open: bool = view.intel.references.is_some() || view.intel.problems_open;
    view.intel.references = None;
    view.intel.problems_open = false;
    open
}

/// Up, Down, Enter and Tab move or accept an open list.
fn list_key(view: &mut ExplorerView, stroke: &Keystroke) -> bool {
    let key: &str = stroke.key.as_str();
    if let Some(popup) = view.intel.completion.as_mut() {
        match key {
            "up" | "down" => popup.step(key == "down"),
            "enter" | "tab" => accept_completion(view),
            "escape" => view.intel.completion = None,
            _ => return false,
        }
        return true;
    }
    if let Some(popup) = view.intel.actions.as_mut() {
        let len: usize = popup.items.len();
        match key {
            "up" | "down" => popup.pick.step(key == "down", len),
            "enter" => {
                let title: String = popup
                    .items
                    .get(popup.pick.selected)
                    .map(|a: &bb_intel::ActionItem| a.title.clone())
                    .unwrap_or_default();
                view.intel.actions = None;
                view.report(&format!("Code action listed only: {title}"));
            }
            "escape" => view.intel.actions = None,
            _ => return false,
        }
        return true;
    }
    false
}

/// Replace the word from the anchor to the cursor with the chosen label.
fn accept_completion(view: &mut ExplorerView) {
    let Some(popup): Option<CompletionPopup> = view.intel.completion.take() else {
        return;
    };
    let Some(label): Option<String> = popup.chosen() else {
        return;
    };
    let Some(doc): Option<&mut Document> = view.tabs.active_mut() else {
        return;
    };
    let cursor: CharIdx = doc.cursor();
    let replaced: Result<(), bb_text::DocumentError> = doc
        .select(popup.anchor, cursor)
        .and_then(|(): ()| doc.insert(&label));
    if let Err(error) = replaced {
        view.report(&error);
    }
}

fn rename_box(view: &mut ExplorerView, stroke: &Keystroke) {
    let Some(input): Option<&mut crate::intel::model::popups::RenameInput> =
        view.intel.rename_input.as_mut()
    else {
        return;
    };
    match stroke.key.as_str() {
        "escape" => view.intel.rename_input = None,
        "enter" => asks::rename_submit(view),
        "backspace" => {
            input.text.pop();
        }
        _ if !stroke.modifiers.platform && !stroke.modifiers.control => {
            if let Some(typed) = stroke.key_char.as_deref() {
                input.text.push_str(typed);
            }
        }
        _ => {}
    }
}

fn preview_key(view: &mut ExplorerView, key: &str) -> bool {
    match key {
        "enter" => {
            match super::apply_rename(view) {
                Ok((files, edits)) => {
                    view.report(&format!("Renamed: {edits} edits in {files} files"));
                }
                Err(error) => view.report(&error),
            }
            true
        }
        "escape" => {
            view.intel.rename_preview = None;
            true
        }
        _ => false,
    }
}

/// After a typed edit: `.` asks for completion; any other text keeps the
/// open list filtered, or closes it once the cursor leaves its word.
pub(crate) fn after_edit(view: &mut ExplorerView, typed: Option<&str>) {
    if typed == Some(".") {
        view.intel.completion = None;
        if view.intel.status == crate::intel::model::LoadStatus::Ready {
            asks::completion(view);
        }
        return;
    }
    let Some(anchor): Option<CharIdx> = view
        .intel
        .completion
        .as_ref()
        .map(|p: &CompletionPopup| p.anchor)
    else {
        return;
    };
    let typed_word: Option<String> = view.tabs.active().and_then(|doc: &Document| {
        (asks::word_start(doc, doc.cursor()) == anchor)
            .then(|| super::typed_since(doc, anchor))
            .flatten()
    });
    match (typed_word, view.intel.completion.as_mut()) {
        (Some(word), Some(popup)) => popup.set_filter(&word),
        (Some(_) | None, _) => view.intel.completion = None,
    }
}
