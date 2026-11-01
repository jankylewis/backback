//! Local C# syntax via tree-sitter.
//!
//! Roslyn is the source of truth for semantics, but it is a separate process and
//! can be seconds away on a big solution. tree-sitter parses incrementally in
//! microseconds, so highlighting, folding and the outline never wait on Roslyn.

mod bb_highlight;
mod live;

pub use bb_highlight::{HighlightKind, HighlightSpan};
pub use live::{LiveTree, ParseJob, Parsed};
pub use tree_sitter::Tree;
use tree_sitter::{Language, Node, Parser, Query, TreeCursor};

/// Syntax errors.
#[derive(Debug, thiserror::Error)]
pub enum SyntaxError {
    /// The grammar could not be loaded into the parser (ABI mismatch).
    #[error("cannot load the C# grammar: {0}")]
    Language(#[from] tree_sitter::LanguageError),
    /// The parse was cancelled or hit the parser's limits.
    #[error("parse did not produce a tree")]
    NoTree,
    /// The parse window was rejected (its bytes are out of order).
    #[error("cannot set the parse window: {0}")]
    Window(#[from] tree_sitter::IncludedRangesError),
    /// The C# highlight query did not compile.
    #[error("cannot load the C# highlight query: {0}")]
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

/// A C# parser. Keep one per buffer and reuse it for incremental parses.
pub struct CSharpSyntax {
    parser: Parser,
    query: Query,
}

impl CSharpSyntax {
    /// Load the C# grammar.
    ///
    /// # Errors
    /// [`SyntaxError::Language`] when the grammar ABI does not match the runtime.
    pub fn new() -> Result<Self, SyntaxError> {
        let mut parser: Parser = Parser::new();
        let language: Language = tree_sitter_c_sharp::LANGUAGE.into();
        parser.set_language(&language)?;
        let query: Query = Query::new(&language, tree_sitter_c_sharp::HIGHLIGHTS_QUERY)?;
        Ok(Self { parser, query })
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
        let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
        let tree: Tree = syntax.parse(src, None).unwrap();
        let items: Vec<OutlineItem> = CSharpSyntax::outline(&tree, src);
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

    #[test]
    fn incremental_reparse_accepts_old_tree() {
        let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
        let first: Tree = syntax.parse("class A {}", None).unwrap();
        let second: Tree = syntax
            .parse("class A { void M() {} }", Some(&first))
            .unwrap();
        assert!(!second.root_node().has_error());
    }
}
