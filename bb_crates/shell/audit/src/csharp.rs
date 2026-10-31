//! C# rules: no `var`, class cap, method cap, parameter cap.

use crate::lexer::{strip, Lang};
use crate::{caps, line_of, matching_brace, Finding, Rule};

const TYPE_WORDS: [&str; 4] = ["class", "struct", "record", "interface"];
const MODIFIERS: [&str; 12] = [
    "public",
    "private",
    "protected",
    "internal",
    "static",
    "async",
    "override",
    "virtual",
    "sealed",
    "abstract",
    "extern",
    "partial",
];

pub(crate) fn check(src: &str) -> Vec<Finding> {
    let text: String = strip(src, Lang::CSharp);
    let mut out: Vec<Finding> = Vec::new();
    check_var(&text, &mut out);
    let mut offset: usize = 0;
    for line in text.split_inclusive('\n') {
        let words: Vec<&str> = line.split_whitespace().collect();
        if is_type_decl(&words) {
            check_block(
                &text,
                offset,
                caps::CS_CLASS_LINES,
                Rule::TypeLines,
                &mut out,
            );
        } else if is_method_decl(line, &words) {
            check_block(
                &text,
                offset,
                caps::FUNCTION_LINES,
                Rule::FunctionLines,
                &mut out,
            );
            check_params(&text, offset, line, &mut out);
        }
        offset = offset.saturating_add(line.len());
    }
    out
}

/// `var x = …`, `var (a, b) = …`, `foreach (var x in …)` are all refused.
fn check_var(text: &str, out: &mut Vec<Finding>) {
    for (at, _) in text.match_indices("var") {
        let before: Option<char> = text.get(..at).and_then(|b: &str| b.chars().next_back());
        let after: &str = text.get(at.saturating_add(3)..).unwrap_or_default();
        let starts_word: bool =
            before.is_none_or(|c: char| !(c.is_alphanumeric() || c == '_' || c == '@'));
        let next: Option<char> = after.trim_start().chars().next();
        let followed: bool = after.starts_with(char::is_whitespace)
            && next.is_some_and(|c: char| c.is_alphabetic() || c == '_' || c == '(');
        if starts_word && followed {
            out.push(Finding {
                line: line_of(text, at),
                rule: Rule::CsVar,
                detail: "`var` hides the type; declare it".to_owned(),
            });
        }
    }
}

fn is_type_decl(words: &[&str]) -> bool {
    let decl: Vec<&&str> = words
        .iter()
        .skip_while(|w: &&&str| MODIFIERS.contains(w))
        .collect();
    decl.first().is_some_and(|w: &&&str| TYPE_WORDS.contains(w)) && !words.contains(&"where")
}

fn is_method_decl(line: &str, words: &[&str]) -> bool {
    let has_modifier: bool = words.first().is_some_and(|w: &&str| MODIFIERS.contains(w));
    let paren: Option<usize> = line.find('(');
    let assign_before: bool =
        paren.is_some_and(|p: usize| line.get(..p).is_some_and(|h: &str| h.contains('=')));
    has_modifier
        && paren.is_some()
        && !assign_before
        && !is_type_decl(words)
        && !line.contains(" new ")
}

fn check_block(text: &str, start: usize, cap: usize, rule: Rule, out: &mut Vec<Finding>) {
    let Some(rest): Option<&str> = text.get(start..) else {
        return;
    };
    let Some(open_rel): Option<usize> = rest.find('{') else {
        return;
    };
    let ends_early: bool = rest.find(';').is_some_and(|s: usize| s < open_rel)
        || rest.find("=>").is_some_and(|a: usize| a < open_rel);
    if ends_early {
        return;
    }
    let Some(open): Option<usize> = start.checked_add(open_rel) else {
        return;
    };
    let Some(close): Option<usize> = matching_brace(text, open) else {
        return;
    };
    let first: usize = line_of(text, start);
    let len: usize = line_of(text, close).saturating_sub(first).saturating_add(1);
    if len > cap {
        out.push(Finding {
            line: first,
            rule,
            detail: format!("{len} lines (max {cap})"),
        });
    }
}

fn check_params(text: &str, start: usize, line: &str, out: &mut Vec<Finding>) {
    let Some(open): Option<usize> = line.find('(') else {
        return;
    };
    let inside: &str = line.get(open.saturating_add(1)..).unwrap_or_default();
    let mut depth: usize = 0;
    let mut count: usize = 0;
    let mut any: bool = false;
    for ch in inside.chars() {
        match ch {
            '(' | '<' | '[' => depth = depth.saturating_add(1),
            ')' if depth == 0 => break,
            ')' | '>' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => count = count.saturating_add(1),
            c if !c.is_whitespace() => any = true,
            _ => {}
        }
    }
    let params: usize = if any { count.saturating_add(1) } else { 0 };
    if params > caps::FUNCTION_PARAMS {
        out.push(Finding {
            line: line_of(text, start),
            rule: Rule::FunctionParams,
            detail: format!("{params} params (max {})", caps::FUNCTION_PARAMS),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(src: &str) -> Vec<Rule> {
        check(src).into_iter().map(|f: Finding| f.rule).collect()
    }

    #[test]
    fn explicit_types_pass_and_var_fails() {
        assert_eq!(rules("public class A { public int M() { int x = 1; string variable = \"var y\"; return x; } }"), Vec::<Rule>::new());
        assert_eq!(
            rules("public class A { void M() { var x = 1; } }"),
            vec![Rule::CsVar]
        );
        assert_eq!(
            rules("public class A { public void M() { foreach (var x in xs) {} } }"),
            vec![Rule::CsVar]
        );
    }

    #[test]
    fn long_class_and_wide_method_are_found() {
        let body: String = "    int f;\n".repeat(caps::CS_CLASS_LINES);
        assert_eq!(
            rules(&format!("public class A\n{{\n{body}}}\n")),
            vec![Rule::TypeLines]
        );
        assert_eq!(
            rules("public class A\n{\n    public void M(int a, int b, int c, int d, int e, int f) { }\n}\n"),
            vec![Rule::FunctionParams]
        );
    }
}
