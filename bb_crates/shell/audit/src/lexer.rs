//! Blank out string literals, char literals and comments while keeping line
//! structure, so structural scans never react to text inside them.

/// Source language, which decides the literal syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// Rust: raw strings `r#"…"#`, lifetimes `'a`.
    Rust,
    /// C#: verbatim strings `@"…"`, `'x'` is always a char literal.
    CSharp,
}

/// `src` with literal contents and comments removed. Quotes stay, newlines stay.
#[must_use]
pub fn strip(src: &str, lang: Lang) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out: String = String::with_capacity(src.len());
    let mut i: usize = 0;
    while let Some(&c) = chars.get(i) {
        let next: Option<char> = chars.get(i.saturating_add(1)).copied();
        let raw_start: bool = lang == Lang::Rust && c == 'r' && !prev_is_ident(&chars, i);
        let verbatim_start: bool = lang == Lang::CSharp && c == '@' && next == Some('"');
        i = if c == '/' && next == Some('/') {
            skip_line_comment(&chars, i)
        } else if c == '/' && next == Some('*') {
            skip_block_comment(&chars, i, &mut out)
        } else if raw_start && matches!(next, Some('#' | '"')) {
            skip_raw(&chars, i, &mut out)
        } else if verbatim_start {
            skip_verbatim(&chars, i.saturating_add(1), &mut out)
        } else if c == '"' {
            skip_string(&chars, i, &mut out)
        } else if c == '\'' {
            skip_char(&chars, i, &mut out, lang)
        } else {
            out.push(c);
            i.saturating_add(1)
        };
    }
    out
}

fn prev_is_ident(chars: &[char], i: usize) -> bool {
    i.checked_sub(1)
        .and_then(|p: usize| chars.get(p))
        .is_some_and(|c: &char| c.is_alphanumeric() || *c == '_')
}

fn skip_line_comment(chars: &[char], mut i: usize) -> usize {
    while chars.get(i).is_some_and(|c: &char| *c != '\n') {
        i = i.saturating_add(1);
    }
    i
}

fn skip_block_comment(chars: &[char], mut i: usize, out: &mut String) -> usize {
    out.push(' ');
    i = i.saturating_add(2);
    while let Some(&c) = chars.get(i) {
        if c == '*' && chars.get(i.saturating_add(1)) == Some(&'/') {
            return i.saturating_add(2);
        }
        if c == '\n' {
            out.push('\n');
        }
        i = i.saturating_add(1);
    }
    i
}

fn skip_string(chars: &[char], mut i: usize, out: &mut String) -> usize {
    out.push('"');
    i = i.saturating_add(1);
    while let Some(&c) = chars.get(i) {
        match c {
            '\\' => i = i.saturating_add(2),
            '"' => {
                out.push('"');
                return i.saturating_add(1);
            }
            '\n' => {
                out.push('\n');
                i = i.saturating_add(1);
            }
            _ => i = i.saturating_add(1),
        }
    }
    i
}

fn skip_verbatim(chars: &[char], mut i: usize, out: &mut String) -> usize {
    out.push('"');
    i = i.saturating_add(1);
    while let Some(&c) = chars.get(i) {
        if c == '"' && chars.get(i.saturating_add(1)) == Some(&'"') {
            i = i.saturating_add(2);
        } else if c == '"' {
            out.push('"');
            return i.saturating_add(1);
        } else {
            if c == '\n' {
                out.push('\n');
            }
            i = i.saturating_add(1);
        }
    }
    i
}

fn skip_raw(chars: &[char], mut i: usize, out: &mut String) -> usize {
    i = i.saturating_add(1);
    let mut hashes: usize = 0;
    while chars.get(i) == Some(&'#') {
        hashes = hashes.saturating_add(1);
        i = i.saturating_add(1);
    }
    if chars.get(i) != Some(&'"') {
        out.push('r');
        return i;
    }
    out.push('"');
    i = i.saturating_add(1);
    while let Some(&c) = chars.get(i) {
        let closes: bool =
            c == '"' && (1..=hashes).all(|k: usize| chars.get(i.saturating_add(k)) == Some(&'#'));
        if closes {
            out.push('"');
            return i.saturating_add(1).saturating_add(hashes);
        }
        if c == '\n' {
            out.push('\n');
        }
        i = i.saturating_add(1);
    }
    i
}

fn skip_char(chars: &[char], i: usize, out: &mut String, lang: Lang) -> usize {
    let next: Option<&char> = chars.get(i.saturating_add(1));
    let after: Option<&char> = chars.get(i.saturating_add(2));
    let is_literal: bool = lang == Lang::CSharp || next == Some(&'\\') || after == Some(&'\'');
    if !is_literal {
        out.push('\''); // a Rust lifetime such as 'a
        return i.saturating_add(1);
    }
    let mut j: usize = i.saturating_add(1);
    while let Some(&c) = chars.get(j) {
        match c {
            '\\' => j = j.saturating_add(2),
            '\'' | '\n' => break,
            _ => j = j.saturating_add(1),
        }
    }
    out.push_str("' '");
    j.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_and_comments_are_blanked() {
        let src: &str = "let a: &str = \"{ let x = 1 }\"; // let y = 2\n/* { */ b";
        let out: String = strip(src, Lang::Rust);
        assert_eq!(out, "let a: &str = \"\"; \n  b");
    }

    #[test]
    fn raw_strings_lifetimes_and_chars() {
        let src: &str = "r#\"a\"b\"# 'a' '{' fn f<'a>(x: &'a str)";
        assert_eq!(strip(src, Lang::Rust), "\"\" ' ' ' ' fn f<'a>(x: &'a str)");
    }

    #[test]
    fn csharp_verbatim_doubles_quotes() {
        assert_eq!(strip("@\"a\"\"{\"; var", Lang::CSharp), "\"\"; var");
    }
}
