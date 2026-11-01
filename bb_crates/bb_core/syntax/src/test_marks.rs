use ropey::Rope;
use tree_sitter::{Node, Tree, TreeCursor};

/// A C# method carrying an NUnit test attribute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestMethod {
    /// Zero-based method declaration line.
    pub line: usize,
    /// Declared method name.
    pub name: String,
}

/// Find test declarations from syntax nodes, excluding comments and strings.
#[must_use]
pub fn test_methods(tree: &Tree, source: &Rope) -> Vec<TestMethod> {
    let mut found: Vec<TestMethod> = Vec::new();
    visit(tree.root_node(), source, &mut found);
    found
}

fn visit(node: Node<'_>, source: &Rope, found: &mut Vec<TestMethod>) {
    if node.kind() == "method_declaration" && has_test_attribute(node, source) {
        if let Some(name) = node
            .child_by_field_name("name")
            .and_then(|name: Node<'_>| node_text(name, source))
        {
            found.push(TestMethod {
                line: node.start_position().row,
                name,
            });
        }
    }
    let mut cursor: TreeCursor<'_> = node.walk();
    for child in node.named_children(&mut cursor) {
        visit(child, source, found);
    }
}

fn has_test_attribute(node: Node<'_>, source: &Rope) -> bool {
    let mut cursor: TreeCursor<'_> = node.walk();
    for list in node.named_children(&mut cursor) {
        if list.kind() != "attribute_list" {
            continue;
        }
        let mut attrs: TreeCursor<'_> = list.walk();
        for attr in list.named_children(&mut attrs) {
            let name: Option<String> = attr
                .child_by_field_name("name")
                .and_then(|name: Node<'_>| node_text(name, source));
            if matches!(
                name.as_deref()
                    .and_then(|name: &str| name.rsplit('.').next()),
                Some("Test" | "TestCase")
            ) {
                return true;
            }
        }
    }
    false
}

fn node_text(node: Node<'_>, source: &Rope) -> Option<String> {
    (node.end_byte() <= source.len_bytes())
        .then(|| source.byte_slice(node.byte_range()).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CSharpSyntax;

    #[test]
    fn finds_test_methods_but_not_comment_text() {
        let source: &str = "class C { // [Test] void Fake() {}\n[Test] void Pass() {}\n[TestCase(1)] void Case(int n) {}\nvoid Plain() {} }";
        let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
        let tree: Tree = syntax.parse(source, None).unwrap();
        assert_eq!(
            test_methods(&tree, &Rope::from_str(source)),
            vec![
                TestMethod {
                    line: 1,
                    name: String::from("Pass")
                },
                TestMethod {
                    line: 2,
                    name: String::from("Case")
                },
            ]
        );
    }
}
