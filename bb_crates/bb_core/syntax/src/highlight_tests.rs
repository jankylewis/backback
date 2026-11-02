//! Highlight and incremental-parse tests (the parse contract: BB-GAT-001).

use super::*;
use bb_text::{CharIdx, Document};
use proptest::prelude::*;

fn reparsed(syntax: &mut CSharpSyntax, doc: &mut Document, tree: &Tree) -> Tree {
    let mut edited: Tree = tree.clone();
    for change in doc.take_changes() {
        CSharpSyntax::edit(&mut edited, &change);
    }
    syntax.parse_rope(doc.rope(), Some(&edited)).unwrap()
}

#[test]
fn edited_tree_and_visible_captures_follow_new_text() {
    let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
    let mut doc: Document = Document::from_text("class A { string S = \"old\"; }", None);
    let first: Tree = syntax.parse_rope(doc.rope(), None).unwrap();
    doc.select(CharIdx::new(22), CharIdx::new(25)).unwrap();
    doc.insert("new").unwrap();
    let next: Tree = reparsed(&mut syntax, &mut doc, &first);
    assert!(!next.root_node().has_error());
    let text: String = doc.text();
    let spans: Vec<HighlightSpan> = syntax.highlight_spans(&next, doc.rope(), 0..text.len());
    assert!(spans
        .iter()
        .any(|span: &HighlightSpan| span.kind == HighlightKind::String
            && text.get(span.start..span.end).unwrap_or("").contains("new")));
    assert!(syntax
        .highlight_spans(&next, doc.rope(), 0..5)
        .iter()
        .all(|span: &HighlightSpan| span.end <= 5));
}

/// Tree-sitter's contract: an incremental reparse equals a fresh parse
/// whenever the text is valid; in broken text its error recovery may
/// keep a different but valid tree for an edit (BB-GAT-001), and the
/// text extent always matches.
fn same_as_fresh(syntax: &mut CSharpSyntax, doc: &Document, tree: &Tree) -> Result<(), String> {
    let fresh: Tree = syntax
        .parse(&doc.text(), None)
        .map_err(|error: crate::SyntaxError| error.to_string())?;
    if tree.root_node().end_position() != fresh.root_node().end_position() {
        return Err(String::from("text extent differs"));
    }
    if !fresh.root_node().has_error() && tree.root_node().to_sexp() != fresh.root_node().to_sexp() {
        return Err(format!(
            "valid text, trees differ:\n{}\n{}",
            tree.root_node().to_sexp(),
            fresh.root_node().to_sexp()
        ));
    }
    Ok(())
}

const SOURCE: &str =
    "namespace N;\r\nclass A\r\n{\r\n    int F() => 1; // c\r\n    string S = \"x\";\r\n}\r\n";

/// CI run 41532817663: proptest's minimal case. The fifth edit (a `{`
/// at the end of broken code) recovers differently; undoing back to the
/// valid source matches a fresh parse exactly again.
#[test]
fn broken_code_may_recover_differently_but_valid_code_always_matches() {
    let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
    let mut doc: Document = Document::from_text(SOURCE, None);
    let mut tree: Tree = syntax.parse_rope(doc.rope(), None).unwrap();
    for (at, enter) in [
        (9_usize, true),
        (129, false),
        (368, false),
        (269, true),
        (253, false),
    ] {
        doc.set_cursor(CharIdx::new(at % (doc.len_chars() + 1)))
            .unwrap();
        if enter {
            doc.enter().unwrap();
        } else {
            doc.insert("{").unwrap();
        }
        tree = reparsed(&mut syntax, &mut doc, &tree);
        same_as_fresh(&mut syntax, &doc, &tree).unwrap();
    }
    for _ in 0..5 {
        doc.undo().unwrap();
        tree = reparsed(&mut syntax, &mut doc, &tree);
    }
    assert_eq!(doc.text(), SOURCE);
    let fresh: Tree = syntax.parse(SOURCE, None).unwrap();
    assert_eq!(tree.root_node().to_sexp(), fresh.root_node().to_sexp());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]
    #[test]
    fn incremental_parse_equals_a_fresh_parse(
        edits in proptest::collection::vec((0usize..400, 0u8..4), 1..25),
    ) {
        let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
        let mut doc: Document = Document::from_text(SOURCE, None);
        let mut tree: Tree = syntax.parse_rope(doc.rope(), None).unwrap();
        for (at, kind) in edits {
            doc.set_cursor(CharIdx::new(at % (doc.len_chars() + 1))).unwrap();
            match kind {
                0 => doc.insert("{").unwrap(),
                1 => doc.backspace().unwrap(),
                2 => doc.enter().unwrap(),
                _ => doc.undo().unwrap(),
            }
            tree = reparsed(&mut syntax, &mut doc, &tree);
            prop_assert_eq!(same_as_fresh(&mut syntax, &doc, &tree), Ok(()));
        }
    }
}
