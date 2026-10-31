//! A syntax tree that keeps up with typing without parsing on the UI thread.
//!
//! Each edit only moves the shown tree's byte ranges (`Tree::edit`, O(depth)).
//! A [`ParseJob`] reparses a rope snapshot on another thread; edits made while
//! it runs are replayed onto its result, so the shown tree always matches the
//! current text's byte offsets.

use bb_text::{Rope, TextChange};
use tree_sitter::Tree;

use crate::{CSharpSyntax, SyntaxError};

/// The shown tree plus the state of the background parse.
pub struct LiveTree {
    tree: Tree,
    parser: Option<CSharpSyntax>,
    during_parse: Vec<TextChange>,
    stale: bool,
}

/// Everything one background parse needs. Send it to a worker and call [`ParseJob::run`].
pub struct ParseJob {
    syntax: CSharpSyntax,
    rope: Rope,
    old: Tree,
}

/// A finished parse, to hand back with [`LiveTree::finish_parse`].
pub struct Parsed {
    syntax: CSharpSyntax,
    tree: Result<Tree, SyntaxError>,
}

impl LiveTree {
    /// Start from a full parse of the current text.
    #[must_use]
    pub fn new(syntax: CSharpSyntax, tree: Tree) -> Self {
        Self {
            tree,
            parser: Some(syntax),
            during_parse: Vec::new(),
            stale: false,
        }
    }

    /// The tree to highlight. Its structure may lag the text; its byte ranges do not.
    #[must_use]
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    /// Whether edits wait for a parse.
    #[must_use]
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// Record edits, oldest first. O(depth) each; never parses.
    pub fn edit(&mut self, changes: &[TextChange]) {
        for change in changes {
            CSharpSyntax::edit(&mut self.tree, change);
        }
        if self.parser.is_none() {
            self.during_parse.extend_from_slice(changes);
        }
        self.stale = self.stale || !changes.is_empty();
    }

    /// Take a parse job if edits wait and no parse runs. `rope` is the current text.
    #[must_use]
    pub fn start_parse(&mut self, rope: &Rope) -> Option<ParseJob> {
        if !self.stale {
            return None;
        }
        let syntax: CSharpSyntax = self.parser.take()?;
        self.stale = false;
        Some(ParseJob {
            syntax,
            rope: rope.clone(),
            old: self.tree.clone(),
        })
    }

    /// Install a finished parse and replay the edits made meanwhile. Returns
    /// the parse error, if any (the shown tree is then kept).
    ///
    /// # Errors
    /// [`SyntaxError::NoTree`] when the background parse produced nothing.
    pub fn finish_parse(&mut self, done: Parsed) -> Result<(), SyntaxError> {
        self.parser = Some(done.syntax);
        let replay: Vec<TextChange> = std::mem::take(&mut self.during_parse);
        let mut tree: Tree = done.tree?;
        for change in &replay {
            CSharpSyntax::edit(&mut tree, change);
        }
        self.tree = tree;
        Ok(())
    }

    /// Parse now on this thread until no edit waits. For tests and small files.
    ///
    /// # Errors
    /// [`SyntaxError::NoTree`] when a parse produced nothing.
    pub fn parse_now(&mut self, rope: &Rope) -> Result<(), SyntaxError> {
        while let Some(job) = self.start_parse(rope) {
            self.finish_parse(job.run())?;
        }
        Ok(())
    }
}

impl ParseJob {
    /// Parse the snapshot incrementally. Slow on a big file: run it off the UI thread.
    #[must_use]
    pub fn run(mut self) -> Parsed {
        let tree: Result<Tree, SyntaxError> = self.syntax.parse_rope(&self.rope, Some(&self.old));
        Parsed {
            syntax: self.syntax,
            tree,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_text::{CharIdx, Document};
    use proptest::prelude::*;

    fn type_at(doc: &mut Document, live: &mut LiveTree, at: usize, op: u8) {
        doc.set_cursor(CharIdx::new(at % (doc.len_chars() + 1)))
            .unwrap();
        match op % 3 {
            0 => doc.insert("{").unwrap(),
            1 => doc.backspace().unwrap(),
            _ => doc.insert("int").unwrap(),
        }
        live.edit(&doc.take_changes());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]
        #[test]
        fn edits_during_a_background_parse_replay_onto_its_result(
            steps in proptest::collection::vec((0usize..300, 0u8..3, any::<bool>()), 1..20),
        ) {
            let source: &str = "class A\n{\n    int F() => 1;\n    string S = \"x\";\n}\n";
            let mut syntax: CSharpSyntax = CSharpSyntax::new().unwrap();
            let mut doc: Document = Document::from_text(source, None);
            let tree: Tree = syntax.parse_rope(doc.rope(), None).unwrap();
            let mut live: LiveTree = LiveTree::new(syntax, tree);
            let mut job: Option<ParseJob> = None;
            for (at, op, toggle) in steps {
                type_at(&mut doc, &mut live, at, op);
                if toggle {
                    match job.take() {
                        Some(running) => live.finish_parse(running.run()).unwrap(),
                        None => job = live.start_parse(doc.rope()),
                    }
                }
                prop_assert_eq!(live.tree().root_node().end_byte(), doc.rope().len_bytes());
            }
            if let Some(running) = job.take() {
                live.finish_parse(running.run()).unwrap();
            }
            live.parse_now(doc.rope()).unwrap();
            prop_assert!(!live.is_stale());
            let fresh: Tree = CSharpSyntax::new().unwrap().parse(&doc.text(), None).unwrap();
            prop_assert_eq!(live.tree().root_node().to_sexp(), fresh.root_node().to_sexp());
        }
    }
}
