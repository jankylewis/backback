//! Local syntax via tree-sitter: C#, and colours for Rust and Python
//! (BBEOD26); a plain-text file parses with no colours.
//!
//! Roslyn is the source of truth for semantics, but it is a separate process and
//! can be seconds away on a big solution. tree-sitter parses incrementally in
//! microseconds, so highlighting, folding and the outline never wait on Roslyn.

mod bb_highlight;
mod live;
mod test_marks;

use bb_highlight::grammar;
pub use bb_highlight::{HighlightKind, HighlightSpan};
use bb_lang::Lang;
pub use live::{LiveTree, ParseJob, Parsed};
pub use test_marks::{test_methods, TestMethod};
pub use tree_sitter::Tree;
use tree_sitter::{Language, Node, Parser, Query, TreeCursor};

/// Syntax errors.
#[derive(Debug, thiserror::Error)]
pub enum SyntaxError {
    /// The grammar could not be loaded into the parser (ABI mismatch).
    #[error("cannot load the grammar: {0}")]
    Language(#[from] tree_sitter::LanguageError),
    /// The parse was cancelled or hit the parser's limits.
    #[error("parse did not produce a tree")]
    NoTree,
    /// The parse window was rejected (its bytes are out of order).
    #[error("cannot set the parse window: {0}")]
    Window(#[from] tree_sitter::IncludedRangesError),
    /// The highlight query did not compile.
    #[error("cannot load the highlight query: {0}")]
    Query(#[from] tree_sitter::QueryError),
}

/// Kind of declaration shown in the outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineKind {
    /// `namespace X` or file-scoped `namespace X;`.
    Namespace,
    /// class, struct, record, interface or enum.
    Type,
    /// method or constructor.
    Member,
}

/// One outline entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineItem {
    /// What it is.
    pub kind: OutlineKind,
    /// Its declared name.
    pub name: String,
    /// 0-based line of the declaration.
    pub line: usize,
    /// Nesting depth (0 = top level).
    pub depth: usize,
}

/// A parser for one language. Keep one per buffer and reuse it for
/// incremental parses.
pub struct Syntax {
    parser: Parser,
    /// None: plain text, no colours.
    query: Option<Query>,
    lang: Option<Lang>,
}

impl Syntax {
    /// Load the C# grammar.
    ///
    /// # Errors
    /// [`SyntaxError::Language`] when the grammar ABI does not match the runtime.
    pub fn new() -> Result<Self, SyntaxError> {
        Self::for_lang(Some(Lang::CSharp))
    }

    /// The grammar and colours of `lang`; None is plain text: a C# parse
    /// (the editor's folding and line math need a tree) with no colours.
    ///
    /// # Errors
    /// [`SyntaxError::Language`] or [`SyntaxError::Query`] when a grammar
    /// does not load.
    pub fn for_lang(lang: Option<Lang>) -> Result<Self, SyntaxError> {
        let (language, highlights): (Language, &str) = grammar(lang.unwrap_or(Lang::CSharp));
        let mut parser: Parser = Parser::new();
        parser.set_language(&language)?;
        let query: Option<Query> = match lang {
            Some(_) => Some(Query::new(&language, highlights)?),
            None => None,
        };
        Ok(Self {
            parser,
            query,
            lang,
        })
    }

    /// The language it colours; None for plain text.
    #[must_use]
    pub fn lang(&self) -> Option<Lang> {
        self.lang
    }

    /// Parse `text`. Pass the previous tree for an incremental parse.
    ///
    /// # Errors
    /// [`SyntaxError::NoTree`] when the parser returns nothing.
    pub fn parse(&mut self, text: &str, old: Option<&Tree>) -> Result<Tree, SyntaxError> {
        self.parser.parse(text, old).ok_or(SyntaxError::NoTree)
    }

    /// Declarations in source order.
    #[must_use]
    pub fn outline(tree: &Tree, text: &str) -> Vec<OutlineItem> {
        let mut out: Vec<OutlineItem> = Vec::new();
        visit(tree.root_node(), text, 0, &mut out);
        out
    }
}

/// The 0-based line of the statement that `line` belongs to, for Set Next
/// Statement (BB-BUG-071): the debugger offers a target only where a
/// statement starts, so a line inside a multi-line statement (an `await`
/// chain, an argument list, a lambda's braces) means that statement.
/// `indent` is the byte column of the line's first non-blank char. `None`
/// when no statement covers the line (a blank line, a brace of the method).
#[must_use]
pub fn statement_line(tree: &Tree, line: usize, indent: usize) -> Option<usize> {
    let at: tree_sitter::Point = tree_sitter::Point::new(line, indent);
    let mut node: Option<Node<'_>> = tree.root_node().descendant_for_point_range(at, at);
    while let Some(here) = node {
        if here.kind().ends_with("_statement") {
            return Some(here.start_position().row);
        }
        node = here.parent();
    }
    None
}

fn kind_of(node: &Node<'_>) -> Option<OutlineKind> {
    match node.kind() {
        "namespace_declaration" | "file_scoped_namespace_declaration" => {
            Some(OutlineKind::Namespace)
        }
        "class_declaration"
        | "struct_declaration"
        | "record_declaration"
        | "interface_declaration"
        | "enum_declaration" => Some(OutlineKind::Type),
        "method_declaration" | "constructor_declaration" => Some(OutlineKind::Member),
        _ => None,
    }
}

fn visit(node: Node<'_>, text: &str, depth: usize, out: &mut Vec<OutlineItem>) {
    let mut child_depth: usize = depth;
    if let Some(kind) = kind_of(&node) {
        let name: Option<&str> = node
            .child_by_field_name("name")
            .and_then(|n: Node<'_>| n.utf8_text(text.as_bytes()).ok());
        if let Some(name) = name {
            out.push(OutlineItem {
                kind,
                name: name.to_owned(),
                line: node.start_position().row,
                depth,
            });
            child_depth = depth.saturating_add(1);
        }
    }
    let mut cursor: TreeCursor<'_> = node.walk();
    for child in node.named_children(&mut cursor) {
        visit(child, text, child_depth, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_lists_namespace_type_and_member() {
        let src: &str = "namespace Demo;\n\npublic class Greeter\n{\n    public string Hello(string name) => $\"hi {name}\";\n}\n";
        let mut syntax: Syntax = Syntax::new().unwrap();
        let tree: Tree = syntax.parse(src, None).unwrap();
        let items: Vec<OutlineItem> = Syntax::outline(&tree, src);
        let shown: Vec<(OutlineKind, &str, usize)> = items
            .iter()
            .map(|i: &OutlineItem| (i.kind, i.name.as_str(), i.line))
            .collect();
        assert_eq!(
            shown,
            vec![
                (OutlineKind::Namespace, "Demo", 0),
                (OutlineKind::Type, "Greeter", 2),
                (OutlineKind::Member, "Hello", 4),
            ]
        );
    }

    /// BB-BUG-071: an async spec as the headmate's, with an `await` chain
    /// over two lines and a lambda with a block body.
    #[test]
    fn a_line_inside_a_statement_means_that_statement() {
        let src: &str = "class S\n{\n    async Task M()\n    {\n        int count = 0;\n        string text = await Task.FromResult(\"a\")\n            .ContinueWith(t => t.Result + count);\n        Func<int, int> add = (int v) =>\n        {\n            return v + count;\n        };\n\n        await Task.Yield();\n    }\n}\n";
        let mut syntax: Syntax = Syntax::new().unwrap();
        let tree: Tree = syntax.parse(src, None).unwrap();
        let at: &dyn Fn(usize) -> Option<usize> = &|line: usize| -> Option<usize> {
            let text: &str = src.lines().nth(line).unwrap();
            let indent: usize = text.len() - text.trim_start().len();
            statement_line(&tree, line, indent)
        };
        assert_eq!(at(4), Some(4), "a statement's own line");
        assert_eq!(at(5), Some(5), "the first line of the await chain");
        assert_eq!(at(6), Some(5), "its continuation means the chain");
        assert_eq!(at(7), Some(7), "the lambda's declaration");
        assert_eq!(at(8), Some(7), "the lambda's brace is that declaration");
        assert_eq!(at(9), Some(9), "a statement of the lambda stays its own");
        assert_eq!(at(10), Some(7), "the closing brace and semicolon");
        assert_eq!(at(11), None, "a blank line");
        assert_eq!(at(12), Some(12));
        assert_eq!(at(3), None, "the method's brace");
    }

    #[test]
    fn incremental_reparse_accepts_old_tree() {
        let mut syntax: Syntax = Syntax::new().unwrap();
        let first: Tree = syntax.parse("class A {}", None).unwrap();
        let second: Tree = syntax
            .parse("class A { void M() {} }", Some(&first))
            .unwrap();
        assert!(!second.root_node().has_error());
    }
}
