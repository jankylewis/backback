//! Rust rules: function and type caps, parameter cap, explicit `let` and
//! closure-parameter types. Scans run on stripped text (no literals, no comments).

use crate::lexer::{strip, Lang};
mod types;

use crate::{caps, line_of, matching_brace, Finding, Rule};

const FN_PREFIXES: [&str; 6] = [
    "pub",
    "pub(crate)",
    "pub(super)",
    "async",
    "const",
    "unsafe",
];
const TYPE_WORDS: [&str; 4] = ["impl", "struct", "enum", "trait"];

pub(crate) fn check(src: &str) -> Vec<Finding> {
    let text: String = strip(src, Lang::Rust);
    let mut out: Vec<Finding> = Vec::new();
    check_blocks(&text, &mut out);
    types::check_lets(&text, &mut out);
    types::check_closures(&text, &mut out);
    out
}

/// Functions (lines + params) and type blocks (lines).
fn check_blocks(text: &str, out: &mut Vec<Finding>) {
    let mut offset: usize = 0;
    for line in text.split_inclusive('\n') {
        let words: Vec<&str> = line
            .split_whitespace()
            .skip_while(|w: &&str| FN_PREFIXES.contains(w))
            .collect();
        let head: Option<&str> = words.first().copied();
        if head == Some("fn") {
            check_fn(text, offset, out);
        } else if head.is_some_and(|w: &str| TYPE_WORDS.contains(&w) || w.starts_with("impl<")) {
            check_type(text, offset, out);
        }
        offset = offset.saturating_add(line.len());
    }
}

fn block_lines(text: &str, start: usize) -> Option<(usize, usize)> {
    let rest: &str = text.get(start..)?;
    let open_rel: usize = rest.find('{')?;
    if rest.find(';').is_some_and(|semi: usize| semi < open_rel) {
        return None; // declaration without a body
    }
    let open: usize = start.checked_add(open_rel)?;
    let close: usize = matching_brace(text, open)?;
    let first: usize = line_of(text, start);
    let last: usize = line_of(text, close);
    Some((first, last.saturating_sub(first).saturating_add(1)))
}

fn check_fn(text: &str, start: usize, out: &mut Vec<Finding>) {
    if let Some((line, len)) = block_lines(text, start) {
        if len > caps::FUNCTION_LINES {
            out.push(Finding {
                line,
                rule: Rule::FunctionLines,
                detail: format!("{len} lines (max {})", caps::FUNCTION_LINES),
            });
        }
    }
    let params: usize = count_params(text.get(start..).unwrap_or_default());
    if params > caps::FUNCTION_PARAMS {
        let line: usize = line_of(text, start);
        out.push(Finding {
            line,
            rule: Rule::FunctionParams,
            detail: format!("{params} params (max {})", caps::FUNCTION_PARAMS),
        });
    }
}

fn check_type(text: &str, start: usize, out: &mut Vec<Finding>) {
    if let Some((line, len)) = block_lines(text, start) {
        if len > caps::RS_TYPE_LINES {
            out.push(Finding {
                line,
                rule: Rule::TypeLines,
                detail: format!("{len} lines (max {})", caps::RS_TYPE_LINES),
            });
        }
    }
}

/// Parameters in the first `(...)` after `fn`, excluding any `self` receiver.
fn count_params(sig: &str) -> usize {
    let Some(open): Option<usize> = sig.find('(') else {
        return 0;
    };
    let mut depth: usize = 0;
    let mut current: String = String::new();
    let mut params: Vec<String> = Vec::new();
    for ch in sig
        .get(open.saturating_add(1)..)
        .unwrap_or_default()
        .chars()
    {
        match ch {
            '(' | '<' | '[' => depth = depth.saturating_add(1),
            ')' if depth == 0 => break,
            ')' | '>' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => params.push(std::mem::take(&mut current)),
            _ => {}
        }
        if !(ch == ',' && depth == 0) {
            current.push(ch);
        }
    }
    params.push(current);
    params
        .iter()
        .map(|p: &String| p.trim())
        .filter(|p: &&str| {
            !p.is_empty()
                && !p
                    .trim_start_matches(['&', ' '])
                    .trim_start_matches("mut ")
                    .starts_with("self")
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(src: &str) -> Vec<Rule> {
        check(src).into_iter().map(|f: Finding| f.rule).collect()
    }

    #[test]
    fn typed_code_passes() {
        let src: &str = "fn f(a: u8) -> u8 { let x: u8 = a; let v: Vec<u8> = vec![1].into_iter().map(|n: u8| n + x).collect(); if let Some(y) = v.first() { return *y; } x }";
        assert_eq!(rules(src), Vec::<Rule>::new());
    }

    #[test]
    fn untyped_let_and_closure_are_found() {
        assert_eq!(rules("fn f() { let x = 1; }"), vec![Rule::LetType]);
        assert_eq!(
            rules("fn f() { let v: Vec<u8> = w.iter().map(|n| n).collect(); }"),
            vec![Rule::ClosureType]
        );
    }

    #[test]
    fn or_operator_and_strings_are_not_closures() {
        assert_eq!(
            rules("fn f(a: bool, b: bool) -> bool { let s: &str = \"|x|\"; a || b }"),
            Vec::<Rule>::new()
        );
    }

    #[test]
    fn long_function_and_many_params_are_found() {
        let body: String = "    let a: u8 = 1;\n".repeat(caps::FUNCTION_LINES);
        assert_eq!(
            rules(&format!("fn f() {{\n{body}}}\n")),
            vec![Rule::FunctionLines]
        );
        assert_eq!(
            rules("fn f(&self, a: u8, b: u8, c: u8, d: u8, e: u8, g: u8) {}"),
            vec![Rule::FunctionParams]
        );
    }
}
