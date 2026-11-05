//! The items of one Rust file, found on stripped text (no literals, no
//! comments) so a string or a comment never reads as an item.

#[path = "head.rs"]
mod head;

use bb_audit::lexer::{strip, Lang};

pub use head::{Kind, Vis};

/// One declared item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// What it is.
    pub kind: Kind,
    /// Its visibility (a trait method takes the trait's).
    pub vis: Vis,
    /// Its name; for an `impl`, the type it is for.
    pub name: String,
    /// 1-based line of its head.
    pub line: usize,
    /// The head on one line, without the body.
    pub signature: String,
    /// The first sentence of its `///` doc.
    pub doc: Option<String>,
    /// The `impl` type or trait it sits in.
    pub owner: Option<String>,
    /// Variant names, for an enum.
    pub variants: Vec<String>,
}

/// One file's map.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileMap {
    /// The first sentence of its `//!` doc.
    pub purpose: Option<String>,
    /// Line count.
    pub lines: usize,
    /// `#[test]`-style functions.
    pub tests: usize,
    /// Items outside function bodies and test modules.
    pub items: Vec<Item>,
}

/// What a `{` opens.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Frame {
    Module { test: bool },
    Owner { name: String, vis: Option<Vis> },
    Enum(usize),
    Body,
}

#[derive(Default)]
struct Scan {
    frames: Vec<(usize, Frame)>,
    depth: usize,
    brackets: usize,
    pending: Option<Frame>,
    cfg_test: bool,
    doc: head::Doc,
    purpose: head::Doc,
    map: FileMap,
}

/// Map one Rust source file.
#[must_use]
pub fn scan(src: &str) -> FileMap {
    let text: String = strip(src, Lang::Rust);
    let code: Vec<&str> = text.lines().collect();
    let original: Vec<&str> = src.lines().collect();
    let mut s: Scan = Scan::default();
    for (index, line) in code.iter().enumerate() {
        let raw: &str = original.get(index).copied().unwrap_or("").trim();
        s.line(index, line, raw, code.get(index..).unwrap_or(&[]));
    }
    s.map.lines = original.len();
    s.map.purpose = s.purpose.take();
    s.map
}

impl Scan {
    fn line(&mut self, index: usize, line: &str, raw: &str, rest: &[&str]) {
        let trimmed: &str = line.trim();
        if let Some(doc) = raw.strip_prefix("//!") {
            if self.depth == 0 {
                self.purpose.push(doc);
            }
            return;
        }
        if let Some(doc) = raw.strip_prefix("///") {
            self.doc.push(doc);
            return;
        }
        if trimmed.starts_with("#[") {
            self.attribute(trimmed);
            return;
        }
        if !trimmed.is_empty() {
            self.code(index, trimmed, rest);
            self.doc = head::Doc::default();
            self.cfg_test = false;
        }
        self.braces(line);
    }

    fn attribute(&mut self, trimmed: &str) {
        let name: &str = trimmed
            .trim_start_matches("#[")
            .split(['(', ']'])
            .next()
            .unwrap_or("");
        if name == "test" || name.ends_with("::test") {
            self.map.tests = self.map.tests.saturating_add(1);
        }
        if trimmed.starts_with("#[cfg(test)]") {
            self.cfg_test = true;
        }
    }

    fn code(&mut self, index: usize, trimmed: &str, rest: &[&str]) {
        if let Some((open, Frame::Enum(at))) = self.frames.last() {
            if self.depth == open.saturating_add(1) {
                let name: &str = head::ident(trimmed);
                if name.starts_with(|c: char| c.is_ascii_uppercase()) {
                    let at: usize = *at;
                    if let Some(item) = self.map.items.get_mut(at) {
                        item.variants.push(name.to_owned());
                    }
                }
            }
            return;
        }
        let open_scope: bool = self.frames.iter().all(|(_, f): &(usize, Frame)| {
            matches!(f, Frame::Module { test: false } | Frame::Owner { .. })
        });
        if !open_scope || self.pending.is_some() {
            return;
        }
        if let Some(found) = head::parse(trimmed) {
            self.found(index, found, rest);
        }
    }

    fn found(&mut self, index: usize, found: head::Head, rest: &[&str]) {
        let (vis, kind, name): (Vis, Kind, String) = (found.vis, found.kind, found.name);
        if kind == Kind::Mod {
            self.pending = Some(Frame::Module {
                test: self.cfg_test,
            });
            return;
        }
        self.pending = Some(match kind {
            Kind::Impl | Kind::Trait => Frame::Owner {
                name: name.clone(),
                vis: (kind == Kind::Trait).then_some(vis),
            },
            Kind::Enum => Frame::Enum(self.map.items.len()),
            Kind::Fn
            | Kind::Struct
            | Kind::Type
            | Kind::Const
            | Kind::Static
            | Kind::Use
            | Kind::Macro
            | Kind::Mod => Frame::Body,
        });
        if self.cfg_test || (kind == Kind::Use && vis == Vis::Private) {
            return;
        }
        let (owner, owner_vis): (Option<String>, Option<Vis>) = self.owner();
        self.map.items.push(Item {
            kind,
            vis: owner_vis.unwrap_or(vis),
            name,
            line: index.saturating_add(1),
            signature: head::signature(rest, kind),
            doc: self.doc.take(),
            owner,
            variants: Vec::new(),
        });
    }

    fn owner(&self) -> (Option<String>, Option<Vis>) {
        match self.frames.last() {
            Some((_, Frame::Owner { name, vis })) => (Some(name.clone()), *vis),
            Some((_, Frame::Module { .. } | Frame::Enum(_) | Frame::Body)) | None => (None, None),
        }
    }

    fn braces(&mut self, line: &str) {
        for c in line.chars() {
            match c {
                '{' => {
                    let frame: Frame = self.pending.take().unwrap_or(Frame::Body);
                    self.frames.push((self.depth, frame));
                    self.depth = self.depth.saturating_add(1);
                }
                '}' => {
                    self.depth = self.depth.saturating_sub(1);
                    if self
                        .frames
                        .last()
                        .is_some_and(|(d, _): &(usize, Frame)| *d == self.depth)
                    {
                        self.frames.pop();
                    }
                }
                '(' | '[' => self.brackets = self.brackets.saturating_add(1),
                ')' | ']' => self.brackets = self.brackets.saturating_sub(1),
                ';' if self.brackets == 0 => self.pending = None,
                _ => {}
            }
        }
    }
}
