//! No vacuous skip in tests: a test must not return early, green, when its
//! tool is missing. A bare `return` (or `return Ok(())`) whose enclosing `if`, `let ... else` or
//! `match` probes for a tool is a violation; the test must fail instead.

use crate::{line_of, Finding, Rule};

/// Calls that ask whether a tool or file is there.
const PROBES: [&str; 8] = [
    "locate(",
    "which(",
    ".exists()",
    ".is_file()",
    ".is_dir()",
    "env::var(",
    "env::var_os(",
    "Command::new(",
];
/// Where the test scope of a source file starts.
const TEST_MARK: &str = "#[cfg(test)]";

/// Scan `text` (stripped); `all_test` when the whole file is test code.
pub(super) fn check(text: &str, all_test: bool, out: &mut Vec<Finding>) {
    let start: usize = if all_test {
        0
    } else {
        match text.find(TEST_MARK) {
            Some(at) => at,
            None => return,
        }
    };
    let mut from: usize = start;
    while let Some(rel) = text.get(from..).and_then(|rest: &str| rest.find("return")) {
        let at: usize = from.saturating_add(rel);
        from = at.saturating_add(1);
        if !is_bare_return(text, at) {
            continue;
        }
        if let Some(probe) = probe_around(text, at) {
            out.push(Finding {
                line: line_of(text, at),
                rule: Rule::VacuousSkip,
                detail: format!("test returns early after `{probe}`; fail instead"),
            });
        }
    }
}

/// `return` as a word with no value, or with a green `Ok(())`.
fn is_bare_return(text: &str, at: usize) -> bool {
    let before: Option<char> = text.get(..at).and_then(|s: &str| s.chars().next_back());
    if before.is_some_and(|c: char| c.is_alphanumeric() || c == '_') {
        return false;
    }
    let after: &str = text
        .get(at.saturating_add(6)..)
        .unwrap_or_default()
        .trim_start();
    after.starts_with("Ok(())") || matches!(after.chars().next(), Some(';' | '}' | ','))
}

/// The probe in the header of the block around `at`, or of the `match`
/// around an arm (`Err(_) => { return; }`).
fn probe_around(text: &str, at: usize) -> Option<&'static str> {
    let mut end: usize = at;
    for _level in 0..2 {
        let open: usize = enclosing_open(text, end)?;
        let header: &str = header_before(text, open);
        if let Some(probe) = PROBES.iter().find(|p: &&&str| header.contains(**p)) {
            return Some(probe);
        }
        if !header.trim_end().ends_with("=>") {
            return None;
        }
        end = open;
    }
    None
}

/// Offset of the `{` that opens the block containing `at`.
fn enclosing_open(text: &str, at: usize) -> Option<usize> {
    let mut depth: usize = 0;
    for (offset, ch) in text.get(..at)?.char_indices().rev() {
        match ch {
            '}' => depth = depth.saturating_add(1),
            '{' if depth == 0 => return Some(offset),
            '{' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

/// Text between the previous statement or block boundary and `open`.
fn header_before(text: &str, open: usize) -> &str {
    let before: &str = text.get(..open).unwrap_or_default();
    let cut: usize = before
        .rfind([';', '{', '}'])
        .map_or(0, |at: usize| at.saturating_add(1));
    before.get(cut..).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(src: &str, all_test: bool) -> Vec<usize> {
        let mut out: Vec<Finding> = Vec::new();
        check(src, all_test, &mut out);
        out.into_iter().map(|f: Finding| f.line).collect()
    }

    #[test]
    fn early_return_on_a_missing_tool_is_found_in_every_shape() {
        let shapes: [&str; 5] = [
            "fn t() -> R {\n if !dotnet.exists() {\n return Ok(());\n }\n Ok(())\n}",
            "fn t() {\n if bb_dap::locate().is_err() {\n return;\n }\n}",
            "fn t() {\n let Ok(p) = which(\"dotnet\") else {\n return;\n };\n}",
            "fn t() {\n match Sdk::locate() {\n Ok(s) => s,\n Err(_) => return,\n };\n}",
            "fn t() {\n match roslyn::locate() {\n Ok(s) => s,\n Err(_) => {\n eprintln!();\n return;\n }\n };\n}",
        ];
        for shape in shapes {
            assert_eq!(found(shape, true).len(), 1, "{shape}");
        }
    }

    #[test]
    fn failing_probes_and_valued_returns_pass() {
        let src: &str = "fn t() {\n let p: P = locate().expect(\"install it\");\n if p.exists() {\n return p;\n }\n if !caps.goto {\n return;\n }\n}";
        assert_eq!(found(src, true), Vec::<usize>::new());
    }

    #[test]
    fn only_test_scope_is_checked() {
        let body: &str = "fn f() {\n if locate().is_err() {\n return;\n }\n}\n";
        assert_eq!(found(body, false), Vec::<usize>::new());
        let tested: String = format!("{body}#[cfg(test)]\nmod tests {{\n{body}}}\n");
        assert_eq!(found(&tested, false), vec![10]);
    }
}
