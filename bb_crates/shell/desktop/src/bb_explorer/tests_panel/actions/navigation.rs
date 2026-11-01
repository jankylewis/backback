use std::path::PathBuf;

use gpui::{Context, ScrollStrategy, Window};

use crate::ExplorerView;

pub(crate) fn source_place(stack: &str) -> Option<(PathBuf, usize)> {
    stack.lines().find_map(|line: &str| {
        let (prefix, one_based): (&str, &str) = line.rsplit_once(":line ")?;
        let (_, file): (&str, &str) = prefix.rsplit_once(" in ")?;
        let line: usize = one_based.trim().parse().ok()?;
        Some((PathBuf::from(file), line.checked_sub(1)?))
    })
}

pub(crate) fn jump(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let source: Option<(PathBuf, usize)> = view
        .tests
        .detail
        .as_ref()
        .and_then(|result: &bb_testkit::SpecResult| result.stack.as_deref())
        .and_then(source_place);
    let (path, line): (PathBuf, usize) = match source {
        Some(source) => source,
        None => return,
    };
    let absolute: PathBuf = if path.is_absolute() {
        path
    } else {
        view.root.join(path)
    };
    let canonical: PathBuf = match absolute.canonicalize() {
        Ok(path) => path,
        Err(_) => return,
    };
    view.pending_test_jump = Some((canonical.clone(), line));
    view.open_file(&canonical, window, cx);
    apply(view, cx);
}

pub(crate) fn apply(view: &mut ExplorerView, cx: &mut Context<ExplorerView>) {
    let (path, line): (&PathBuf, &usize) = match &view.pending_test_jump {
        Some((path, line)) => (path, line),
        None => return,
    };
    let tab: &crate::editor::Tab = match view.tabs.active_tab() {
        Some(tab) => tab,
        None => return,
    };
    if tab.path != *path {
        return;
    }
    let target: Option<bb_text::CharIdx> = tab.document.line_start(*line);
    if let (Some(at), Some(doc)) = (target, view.tabs.active_mut()) {
        if doc.set_cursor(at).is_ok() {
            view.editor_scroll
                .scroll_to_item(*line, ScrollStrategy::Center);
            view.pending_test_jump = None;
            cx.notify();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stack_source_uses_the_file_and_one_based_line() {
        let stack: &str = "at N.C.Test() in /repo/src/A.cs:line 12\nat N.C.Other()";
        assert_eq!(
            source_place(stack),
            Some((PathBuf::from("/repo/src/A.cs"), 11))
        );
    }
}
