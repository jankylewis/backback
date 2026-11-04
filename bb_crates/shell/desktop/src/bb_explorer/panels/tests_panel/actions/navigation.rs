use std::path::PathBuf;

use gpui::{Context, ScrollStrategy, Window};

use crate::ExplorerView;

/// The first stack frame with a source place.
pub(crate) fn source_place(stack: &str) -> Option<(PathBuf, usize)> {
    stack.lines().find_map(frame_place)
}

/// One stack-trace line's source place, 0-based line (BBEOD18), from the
/// .NET shape `at N.C.M(args) in <path>:line <n>`. The path starts after the
/// method's `) in ` (so a folder named `x in y` stays whole) and ends at the
/// last `:line `; a trailing `\r` is trimmed. Frames without source give
/// `None`.
pub(crate) fn frame_place(line: &str) -> Option<(PathBuf, usize)> {
    let line: &str = line.trim_end_matches(['\r', '\n']);
    let (prefix, one_based): (&str, &str) = line.rsplit_once(":line ")?;
    let (_, file): (&str, &str) = prefix.split_once(") in ")?;
    let number: usize = one_based.trim().parse().ok()?;
    let file: &str = file.trim();
    if file.is_empty() {
        return None;
    }
    Some((PathBuf::from(file), number.checked_sub(1)?))
}

pub(crate) fn jump(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    let source: Option<(PathBuf, usize)> = view
        .tests
        .detail
        .as_ref()
        .and_then(|result: &bb_testkit::SpecResult| result.stack.as_deref())
        .and_then(source_place);
    if let Some((path, line)) = source {
        jump_to(view, (&path, line), window, cx);
    }
}

/// Open a stack frame's file at its 0-based line. Relative paths and a
/// deterministic build's `/_/` paths are read under the opened root; a file
/// not on this machine is said in the status bar.
pub(crate) fn jump_to(
    view: &mut ExplorerView,
    (path, line): (&std::path::Path, usize),
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let local: PathBuf = match path.strip_prefix("/_/") {
        Ok(rest) => view.root.join(rest),
        Err(_) if path.is_absolute() => path.to_path_buf(),
        Err(_) => view.root.join(path),
    };
    let Ok(canonical): Result<PathBuf, std::io::Error> = local.canonicalize() else {
        view.report(&format!(
            "Cannot open {}: not on this machine",
            path.display()
        ));
        return;
    };
    view.tests.pending_jump = Some((canonical.clone(), line));
    view.open_file(&canonical, window, cx);
    apply(view, cx);
}

pub(crate) fn apply(view: &mut ExplorerView, cx: &mut Context<ExplorerView>) {
    let (path, line): (&PathBuf, &usize) = match &view.tests.pending_jump {
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
            view.tabs
                .scroll
                .scroll_to_item(*line, ScrollStrategy::Center);
            view.tests.pending_jump = None;
            cx.notify();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn place(line: &str) -> Option<(String, usize)> {
        frame_place(line).map(|(p, l): (PathBuf, usize)| (p.display().to_string(), l))
    }

    #[test]
    fn every_dotnet_frame_shape_gives_its_file_and_line() {
        let cases: [(&str, &str, usize); 7] = [
            (
                "   at N.C.Test() in /repo/src/A.cs:line 12",
                "/repo/src/A.cs",
                11,
            ),
            ("at N.C.M[T](T x, Int32 y) in /r/G.cs:line 3", "/r/G.cs", 2),
            ("   at N.C.<Main>b__0_0() in /r/L.cs:line 9\r", "/r/L.cs", 8),
            (
                "   at N.C.<Run>d__4.MoveNext() in /r/Async.cs:line 40",
                "/r/Async.cs",
                39,
            ),
            (
                "   at N.C.M() in /srv/my in folder/A.cs:line 5",
                "/srv/my in folder/A.cs",
                4,
            ),
            (
                "   at N.C.M() in C:\\src\\App\\A.cs:line 7",
                "C:\\src\\App\\A.cs",
                6,
            ),
            (
                "   at N.C.M() in /_/src/Core/A.cs:line 2",
                "/_/src/Core/A.cs",
                1,
            ),
        ];
        for (line, file, at) in cases {
            assert_eq!(place(line), Some((file.to_owned(), at)), "{line}");
        }
    }

    #[test]
    fn frames_without_source_and_other_text_give_nothing() {
        for line in [
            "   at System.Threading.Tasks.Task.Run()",
            "Expected: 3 But was: 2",
            "   at N.C.M() in /r/A.cs:line x",
            "   at N.C.M() in /r/A.cs:line 0",
            "",
        ] {
            assert_eq!(place(line), None, "{line}");
        }
    }

    #[test]
    fn stack_source_uses_the_file_and_one_based_line() {
        let stack: &str = "at N.C.Test() in /repo/src/A.cs:line 12\nat N.C.Other()";
        assert_eq!(
            source_place(stack),
            Some((PathBuf::from("/repo/src/A.cs"), 11))
        );
    }
}
