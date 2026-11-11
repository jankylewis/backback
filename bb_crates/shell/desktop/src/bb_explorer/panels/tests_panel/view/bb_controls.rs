use crate::ExplorerView;
use gpui::{Context, KeyDownEvent, Window};

/// Keys of the filter and the category boxes. In the filter, Enter and a
/// paste also search the projects not listed yet.
pub(super) fn filter_key(
    view: &mut ExplorerView,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let category: bool = view.tests.input == super::super::Input::Category;
    if view.tests.input == super::super::Input::None {
        return;
    }
    let pasted: Option<String> = pasted(event, cx);
    let text: &mut String = if category {
        &mut view.tests.category
    } else {
        &mut view.tests.filter
    };
    let mut find: bool = false;
    match event.keystroke.key.as_str() {
        "backspace" if event.keystroke.modifiers.platform => text.clear(),
        "backspace" => {
            text.pop();
        }
        "escape" => {
            view.tests.input = super::super::Input::None;
        }
        "enter" => {
            view.tests.input = super::super::Input::None;
            find = !category;
            if category {
                super::super::run_category(view, window, cx);
            }
        }
        _ => {
            if let Some(line) = &pasted {
                text.push_str(line);
                find = !category;
            } else if let Some(ch) = &event.keystroke.key_char {
                text.push_str(ch);
            }
        }
    }
    cx.stop_propagation();
    if find {
        super::super::find::search(view, window, cx);
    }
    cx.notify();
}

/// Cmd+V: the clipboard's first line, trimmed (a test's name copied in the
/// editor may bring spaces or a line break).
fn pasted(event: &KeyDownEvent, cx: &mut Context<ExplorerView>) -> Option<String> {
    let paste: bool = event.keystroke.key == "v" && event.keystroke.modifiers.platform;
    let text: String = paste.then(|| cx.read_from_clipboard()).flatten()?.text()?;
    text.lines().next().map(|line: &str| line.trim().to_owned())
}
