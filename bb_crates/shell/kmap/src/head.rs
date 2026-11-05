//! Reading one item head: visibility, keyword, name, one-line signature, and
//! the first sentence of a doc comment.

/// Longest signature kept, in chars.
const SIGNATURE_CHARS: usize = 160;
/// Longest doc sentence kept, in chars.
const DOC_CHARS: usize = 100;
/// Most lines one signature may span.
const SIGNATURE_LINES: usize = 12;

/// What an item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `fn`.
    Fn,
    /// `struct`.
    Struct,
    /// `enum`.
    Enum,
    /// `trait`.
    Trait,
    /// `type` alias.
    Type,
    /// `const`.
    Const,
    /// `static`.
    Static,
    /// `impl` block; its name is the type it is for.
    Impl,
    /// `pub use` re-export.
    Use,
    /// `macro_rules!`.
    Macro,
    /// `mod` (a scope, never listed).
    Mod,
}

/// How far an item is seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Vis {
    /// No `pub`.
    Private,
    /// `pub(crate)`, `pub(super)`, `pub(in ..)`.
    Restricted,
    /// `pub`.
    Public,
}

/// A parsed head.
pub(super) struct Head {
    pub(super) vis: Vis,
    pub(super) kind: Kind,
    pub(super) name: String,
}

const QUALIFIERS: [&str; 4] = ["async ", "unsafe ", "default ", "extern \"\" "];
const KEYWORDS: [(&str, Kind); 11] = [
    ("fn", Kind::Fn),
    ("struct", Kind::Struct),
    ("enum", Kind::Enum),
    ("trait", Kind::Trait),
    ("type", Kind::Type),
    ("const", Kind::Const),
    ("static", Kind::Static),
    ("impl", Kind::Impl),
    ("use", Kind::Use),
    ("macro_rules", Kind::Macro),
    ("mod", Kind::Mod),
];

/// The head on a stripped, trimmed line, if the line starts an item.
pub(super) fn parse(line: &str) -> Option<Head> {
    let (vis, mut rest): (Vis, &str) = split_vis(line)?;
    while let Some(q) = QUALIFIERS.iter().find(|q: &&&str| rest.starts_with(**q)) {
        rest = rest.get(q.len()..).unwrap_or("");
    }
    if let Some(after) = rest.strip_prefix("const ") {
        if after.starts_with("fn ") || after.starts_with("unsafe ") {
            rest = after.trim_start_matches("unsafe ");
        }
    }
    let word: &str = ident(rest);
    let kind: Kind = KEYWORDS.iter().find(|(w, _): &&(&str, Kind)| *w == word)?.1;
    let after: &str = rest.get(word.len()..).unwrap_or("");
    let name: String = match kind {
        Kind::Impl => owner_of(after),
        Kind::Use => "use".to_owned(),
        Kind::Macro => ident(after.trim_start_matches('!').trim_start()).to_owned(),
        Kind::Static => ident(after.trim_start().trim_start_matches("mut ")).to_owned(),
        Kind::Fn
        | Kind::Struct
        | Kind::Enum
        | Kind::Trait
        | Kind::Type
        | Kind::Const
        | Kind::Mod => ident(after.trim_start()).to_owned(),
    };
    let spaced: bool = after.starts_with([' ', '<', '!', '{']);
    (spaced && !name.is_empty()).then_some(Head { vis, kind, name })
}

fn split_vis(line: &str) -> Option<(Vis, &str)> {
    if let Some(rest) = line.strip_prefix("pub(") {
        let close: usize = rest.find(')')?;
        let after: &str = rest.get(close.saturating_add(1)..)?;
        return Some((Vis::Restricted, after.trim_start()));
    }
    Some(
        line.strip_prefix("pub ")
            .map_or((Vis::Private, line), |r: &str| (Vis::Public, r)),
    )
}

/// The leading identifier of `text`.
pub(super) fn ident(text: &str) -> &str {
    let end: usize = text
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(text.len());
    text.get(..end).unwrap_or("")
}

/// `impl<T> Trait<X> for Type<T>` → `Type`; `impl Type` → `Type`.
fn owner_of(after: &str) -> String {
    let mut rest: &str = after.trim_start();
    if rest.starts_with('<') {
        rest = rest.get(closing_angle(rest)..).unwrap_or("").trim_start();
    }
    if let Some((_, target)) = rest.rsplit_once(" for ") {
        rest = target.trim_start();
    }
    let path: &str = rest
        .trim_start_matches('&')
        .trim_start_matches("dyn ")
        .split(['<', ' ', '{'])
        .next()
        .unwrap_or("");
    path.rsplit("::").next().unwrap_or(path).to_owned()
}

fn closing_angle(text: &str) -> usize {
    let mut depth: usize = 0;
    for (at, c) in text.char_indices() {
        match c {
            '<' => depth = depth.saturating_add(1),
            '>' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return at.saturating_add(1);
                }
            }
            _ => {}
        }
    }
    text.len()
}

/// The head from its first line to the body or `;`, on one line.
pub(super) fn signature(lines: &[&str], kind: Kind) -> String {
    let mut out: String = String::new();
    let mut brackets: usize = 0;
    'lines: for line in lines.iter().take(SIGNATURE_LINES) {
        if !out.is_empty() && !out.ends_with(['(', '[']) {
            out.push(' ');
        }
        for c in line.trim().chars() {
            match c {
                '(' | '[' | '<' => brackets = brackets.saturating_add(1),
                ')' | ']' | '>' => brackets = brackets.saturating_sub(1),
                '{' | ';' if brackets == 0 && kind != Kind::Use => break 'lines,
                ';' if brackets == 0 => break 'lines,
                '=' if brackets == 0 && matches!(kind, Kind::Const | Kind::Static) => break 'lines,
                _ => {}
            }
            out.push(c);
        }
    }
    let one: String = out.split_whitespace().collect::<Vec<&str>>().join(" ");
    let one: String = one
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(", )", ")");
    cut(one.trim_end_matches(',').trim(), SIGNATURE_CHARS)
}

fn cut(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut kept: String = text.chars().take(max).collect();
    kept.push('…');
    kept
}

/// A doc comment read line by line, kept up to its first sentence.
#[derive(Debug, Default)]
pub(super) struct Doc {
    text: String,
    done: bool,
}

impl Doc {
    /// Add one doc line (the text after `///` or `//!`).
    pub(super) fn push(&mut self, line: &str) {
        let line: &str = line.trim();
        if self.done || (line.is_empty() && !self.text.is_empty()) {
            self.done = true;
            return;
        }
        if !self.text.is_empty() {
            self.text.push(' ');
        }
        let end: Option<usize> = line.find(". ").map(|at: usize| at.saturating_add(1));
        self.text
            .push_str(end.and_then(|e: usize| line.get(..e)).unwrap_or(line));
        self.done = end.is_some() || line.ends_with('.');
    }

    /// The sentence, if any; leaves the doc empty.
    pub(super) fn take(&mut self) -> Option<String> {
        let text: String = std::mem::take(&mut self.text);
        self.done = false;
        (!text.is_empty()).then(|| cut(text.trim(), DOC_CHARS))
    }
}
