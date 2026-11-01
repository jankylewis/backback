use crate::ExplorerView;
use gpui::{Context, KeyDownEvent, Window};

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
    let text: &mut String = if category {
        &mut view.tests.category
    } else {
        &mut view.tests.filter
    };
    match event.keystroke.key.as_str() {
        "backspace" => {
            text.pop();
        }
        "escape" => {
            view.tests.input = super::super::Input::None;
        }
        "enter" => {
            view.tests.input = super::super::Input::None;
            if category {
                super::super::run_category(view, window, cx);
            }
        }
        _ => {
            if let Some(ch) = &event.keystroke.key_char {
                text.push_str(ch);
            }
        }
    }
    cx.stop_propagation();
    cx.notify();
}
