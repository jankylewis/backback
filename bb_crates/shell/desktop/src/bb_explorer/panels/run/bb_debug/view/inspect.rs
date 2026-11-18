//! Looking into values while paused: open a variable to see its children
//! (BB-FEAT-087), rest the mouse on a name to see its value (BB-FEAT-088),
//! as in Rider. A name the debugger cannot evaluate gets the server's hover.

use std::path::Path;

use bb_text::CharIdx;

use crate::intel::model::popups::HoverPopup;
use crate::run::bb_debug::session::control::{Live, Phase};
use crate::run::bb_debug::session::{Command, Frame, Var};
use crate::ExplorerView;

/// Ask for the children of every open row not asked yet at this stop.
pub(crate) fn ask_children(view: &mut ExplorerView) {
    let Some(live): Option<&Live> = view.debug.live.as_ref() else {
        return;
    };
    let vars: &Vec<Var> = match &live.phase {
        Phase::Stopped { vars, .. } => vars,
        Phase::Building | Phase::Running => return,
    };
    let wanted: Vec<i64> = view.debug.tree.wanted(vars);
    for reference in wanted {
        live.send(Command::Expand(reference));
    }
}

/// An opened row's children arrived.
pub(crate) fn children(view: &mut ExplorerView, reference: i64, vars: Vec<Var>) {
    view.debug.tree.loaded(reference, vars);
    ask_children(view);
}

/// The mouse rests on `at` of `path`: while paused in that file, ask the
/// debugger for the value of the name there. False when not asked (then
/// the server's hover runs).
pub(crate) fn hover_value(view: &mut ExplorerView, path: &Path, at: CharIdx, token: u64) -> bool {
    let stopped_here: bool = super::notes::stopped_at(view)
        .is_some_and(|(file, _): (std::path::PathBuf, usize)| file == path);
    if !stopped_here {
        return false;
    }
    let Some(frame): Option<i64> = selected_frame(view) else {
        return false;
    };
    let Some(expression): Option<String> = view
        .tabs
        .tab_mut(path)
        .and_then(|tab: &mut crate::editor::Tab| expression_at(&tab.document, at))
    else {
        return false;
    };
    view.debug.live.as_ref().is_some_and(|live: &Live| {
        live.send(Command::Evaluate {
            expression,
            frame,
            token,
        })
    })
}

fn selected_frame(view: &ExplorerView) -> Option<i64> {
    match &view.debug.live.as_ref()?.phase {
        Phase::Stopped {
            frames, selected, ..
        } => frames.get(*selected).map(|f: &Frame| f.id),
        Phase::Building | Phase::Running => None,
    }
}

/// A value came back: shown while the mouse still rests where it was
/// asked; no value hands the hover to the server.
pub(crate) fn evaluated(view: &mut ExplorerView, token: u64, value: Result<Var, String>) {
    let Some((path, at, _)): Option<(std::path::PathBuf, CharIdx, u64)> = view
        .intel
        .hover_target
        .clone()
        .filter(|target: &(std::path::PathBuf, CharIdx, u64)| target.2 == token)
    else {
        return;
    };
    let text: String = match value {
        Ok(var) => format!("{} = {}", var.name, var.value),
        // Not in the running build: never the server's type (BB-BUG-072).
        Err(_) if super::bb_panel::stale::file_changed(view, &path) => {
            format!("no value: {}", super::bb_panel::stale::NOTICE)
        }
        Err(_) => return crate::intel::glue::hover_ask(view, path, at),
    };
    let line: u32 = view
        .tabs
        .tab_mut(&path)
        .map_or(0, |tab: &mut crate::editor::Tab| {
            let line: usize = tab.document.rope().char_to_line(at.raw());
            u32::try_from(line).unwrap_or(0)
        });
    view.intel.hover = Some(HoverPopup {
        text,
        line,
        x: view.intel.hover_at.0,
        y: view.intel.hover_at.1,
    });
}

/// The name under `at`, with the `a.b.` before it: `in_body`, `x.Count`.
/// None off a name, on a keyword, or on a call (`Foo(`).
pub(crate) fn expression_at(doc: &bb_text::Document, at: CharIdx) -> Option<String> {
    let index: usize = at.raw();
    let line: usize = doc.rope().char_to_line(index);
    let column: usize = index.checked_sub(doc.rope().line_to_char(line))?;
    expression_in(&doc.line(line)?, column)
}

/// [`expression_at`] within one line, `column` in chars.
pub(crate) fn expression_in(line: &str, column: usize) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    let word: fn(&char) -> bool = |c: &char| c.is_alphanumeric() || *c == '_';
    if !chars.get(column).is_some_and(word) {
        return None;
    }
    let mut end: usize = column;
    while chars.get(end.saturating_add(1)).is_some_and(word) {
        end = end.saturating_add(1);
    }
    let mut start: usize = column;
    loop {
        while start > 0 && chars.get(start.saturating_sub(1)).is_some_and(word) {
            start = start.saturating_sub(1);
        }
        let dotted: bool = start >= 2
            && chars.get(start.saturating_sub(1)) == Some(&'.')
            && chars.get(start.saturating_sub(2)).is_some_and(word);
        if !dotted {
            break;
        }
        start = start.saturating_sub(1);
    }
    let after: Option<&char> = chars
        .get(end.saturating_add(1)..)
        .and_then(|rest: &[char]| rest.iter().find(|c: &&char| !c.is_whitespace()));
    let text: String = chars.get(start..=end)?.iter().collect();
    let first: char = text.chars().next()?;
    if after == Some(&'(') || first.is_numeric() || KEYWORDS.contains(&text.as_str()) {
        return None;
    }
    Some(text)
}

/// C# words that are not values.
const KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "async",
    "await",
    "bool",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "false",
    "finally",
    "for",
    "foreach",
    "if",
    "in",
    "int",
    "interface",
    "internal",
    "is",
    "long",
    "namespace",
    "new",
    "null",
    "object",
    "out",
    "override",
    "private",
    "protected",
    "public",
    "readonly",
    "ref",
    "return",
    "sealed",
    "static",
    "string",
    "struct",
    "switch",
    "throw",
    "true",
    "try",
    "typeof",
    "using",
    "var",
    "virtual",
    "void",
    "while",
];

#[cfg(test)]
mod tests {
    use super::expression_in;

    #[test]
    fn a_name_under_the_mouse_is_its_expression() {
        let line: &str = "    _Line(\"ACTION: \" + in_name);";
        assert_eq!(expression_in(line, 26).as_deref(), Some("in_name"));
        assert_eq!(
            expression_in(line, 29).as_deref(),
            Some("in_name"),
            "last char"
        );
        assert_eq!(expression_in(line, 22), None, "the space before");
    }

    #[test]
    fn a_member_carries_the_names_before_its_dot() {
        let line: &str = "var n = page.Options.Count;";
        assert_eq!(
            expression_in(line, 22).as_deref(),
            Some("page.Options.Count")
        );
        assert_eq!(expression_in(line, 9).as_deref(), Some("page"));
    }

    #[test]
    fn keywords_calls_numbers_and_spaces_have_no_value() {
        let line: &str = "await Foo(in_body, 42);";
        assert_eq!(expression_in(line, 1), None, "keyword");
        assert_eq!(expression_in(line, 7), None, "call");
        assert_eq!(expression_in(line, 20), None, "number");
        assert_eq!(expression_in(line, 5), None, "space");
        assert_eq!(expression_in(line, 12).as_deref(), Some("in_body"));
    }
}
