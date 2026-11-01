//! JSON with comments and trailing commas (VS Code `settings.json`).

/// `text` as plain JSON: `//` and `/* */` comments outside strings become
/// spaces (line breaks kept), and a comma before `}` or `]` is dropped.
/// Text inside strings, escapes included, is never touched.
#[must_use]
pub fn strip_jsonc(text: &str) -> String {
    let no_comments: String = strip_comments(text);
    strip_trailing_commas(&no_comments)
}

/// Where the scanner is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Code,
    Text,
    TextEscape,
    LineComment,
    BlockComment,
}

fn strip_comments(text: &str) -> String {
    let mut out: String = String::with_capacity(text.len());
    let mut mode: Mode = Mode::Code;
    let mut chars: std::iter::Peekable<std::str::Chars<'_>> = text.chars().peekable();
    while let Some(c) = chars.next() {
        let next: Option<char> = chars.peek().copied();
        mode = match (mode, c, next) {
            (Mode::Code, '/', Some('/')) => {
                chars.next();
                out.push_str("  ");
                Mode::LineComment
            }
            (Mode::Code, '/', Some('*')) => {
                chars.next();
                out.push_str("  ");
                Mode::BlockComment
            }
            (Mode::LineComment, '\n', _) => {
                out.push('\n');
                Mode::Code
            }
            (Mode::BlockComment, '*', Some('/')) => {
                chars.next();
                out.push_str("  ");
                Mode::Code
            }
            (Mode::LineComment | Mode::BlockComment, _, _) => {
                out.push(if c == '\n' { '\n' } else { ' ' });
                mode
            }
            (Mode::Code, '"', _) => keep(&mut out, c, Mode::Text),
            (Mode::Text, '\\', _) => keep(&mut out, c, Mode::TextEscape),
            (Mode::Text, '"', _) | (Mode::TextEscape, _, _) => keep(
                &mut out,
                c,
                if mode == Mode::Text {
                    Mode::Code
                } else {
                    Mode::Text
                },
            ),
            (Mode::Code | Mode::Text, _, _) => keep(&mut out, c, mode),
        };
    }
    out
}

fn keep(out: &mut String, c: char, next: Mode) -> Mode {
    out.push(c);
    next
}

fn strip_trailing_commas(text: &str) -> String {
    let mut out: String = String::with_capacity(text.len());
    let mut in_text: bool = false;
    let mut escaped: bool = false;
    let mut pending: Option<String> = None;
    for c in text.chars() {
        if in_text {
            in_text = escaped || c != '"';
            escaped = !escaped && c == '\\';
            out.push(c);
            continue;
        }
        if let Some(held) = pending.as_mut() {
            if c.is_whitespace() {
                held.push(c);
                continue;
            }
            let held: String = pending.take().unwrap_or_default();
            if c != '}' && c != ']' {
                out.push(',');
            }
            out.push_str(&held);
        }
        match c {
            ',' => pending = Some(String::new()),
            '"' => {
                in_text = true;
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    if let Some(held) = pending {
        out.push(',');
        out.push_str(&held);
    }
    out
}
