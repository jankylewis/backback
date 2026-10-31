//! Explicit types: every `let` declares its type, every closure parameter too.
//! `if let`, `while let`, `match` and `for` patterns are exempt (the grammar has
//! no place for an annotation there).

use crate::{line_of, Finding, Rule};

/// Every `let` (not `if let` / `while let`) declares a type before its `=`.
pub(crate) fn check_lets(text: &str, out: &mut Vec<Finding>) {
    for (at, _) in text.match_indices("let ") {
        let before: &str = text.get(..at).unwrap_or_default();
        let boundary: bool = before
            .chars()
            .next_back()
            .is_none_or(|c: char| !(c.is_alphanumeric() || c == '_'));
        let prev_word: Option<&str> = before.split_whitespace().next_back();
        if !boundary || matches!(prev_word, Some("if" | "while" | "&&")) {
            continue;
        }
        let binding: &str = binding_text(text.get(at.saturating_add(4)..).unwrap_or_default());
        if !binding.replace("::", "").contains(':') {
            let shown: String = binding.split_whitespace().collect::<Vec<&str>>().join(" ");
            out.push(Finding {
                line: line_of(text, at),
                rule: Rule::LetType,
                detail: format!("`let {shown}` has no type"),
            });
        }
    }
}

/// Text of a `let` binding up to its `=` (or `;` for `let x: T;`).
fn binding_text(rest: &str) -> &str {
    let chars: Vec<(usize, char)> = rest.char_indices().collect();
    for (pos, &(idx, c)) in chars.iter().enumerate() {
        let next: Option<char> = chars
            .get(pos.saturating_add(1))
            .map(|t: &(usize, char)| t.1);
        let prev: Option<char> = pos
            .checked_sub(1)
            .and_then(|p: usize| chars.get(p))
            .map(|t: &(usize, char)| t.1);
        let is_assign: bool = c == '='
            && !matches!(next, Some('=' | '>'))
            && !matches!(prev, Some('<' | '>' | '!' | '='));
        if is_assign || c == ';' {
            return rest.get(..idx).unwrap_or(rest);
        }
    }
    rest
}

/// Every closure parameter declares a type: `|x: T|`, never `|x|`.
pub(crate) fn check_closures(text: &str, out: &mut Vec<Finding>) {
    for (at, _) in text.match_indices('|') {
        let before: &str = text.get(..at).unwrap_or_default().trim_end();
        let opens: bool = before.ends_with(['(', ',', '=', '{'])
            || before.ends_with("move")
            || before.ends_with("return");
        let after: &str = text.get(at.saturating_add(1)..).unwrap_or_default();
        if !opens || after.starts_with('|') || before.ends_with('|') {
            continue;
        }
        let Some(end): Option<usize> = after.find('|') else {
            continue;
        };
        let params: &str = after.get(..end).unwrap_or_default();
        let untyped: bool = split_top(params)
            .iter()
            .any(|p: &String| !p.trim().is_empty() && !p.contains(':'));
        if untyped {
            out.push(Finding {
                line: line_of(text, at),
                rule: Rule::ClosureType,
                detail: format!("closure `|{}|` has an untyped parameter", params.trim()),
            });
        }
    }
}

fn split_top(params: &str) -> Vec<String> {
    let mut depth: usize = 0;
    let mut parts: Vec<String> = vec![String::new()];
    for ch in params.chars() {
        match ch {
            '(' | '<' | '[' => depth = depth.saturating_add(1),
            ')' | '>' | ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if ch == ',' && depth == 0 {
            parts.push(String::new());
        } else if let Some(last) = parts.last_mut() {
            last.push(ch);
        }
    }
    parts
}
