//! Property: after any sequence of incremental edits, the text the server
//! holds equals the bb-text mirror.
//!
//! The witness is independent of the mirror code: the server's semantic
//! tokens and format edits for the document, asked once after the
//! incremental edits and once after a full-text resync of the mirror. Tokens
//! carry every token's line, UTF-16 column and length; format edits carry the
//! whitespace. A drifted server text gives different answers.

#[cfg(test)]
mod common;

use std::cell::RefCell;

use bb_intel::DocPath;
use common::{start, Harness};
use props::{one_case, raw_edits, RawEdit};
use proptest::test_runner::{Config, TestRunner};

#[cfg(test)]
mod props {
    use crate::common::DEADLINE;
    use bb_intel::{DocPath, DocVersion, Edit, Session};
    use bb_text::CharIdx;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseError;
    use serde_json::{json, Value};

    const PIECES: [&str; 16] = [
        "x", "int ", " ", "\n", "\r\n", "\r", "é", "😀", ";", "{", "}", "//", "\"", "\u{2028}",
        "\u{85}", "\u{2029}",
    ];

    /// One raw edit: a start and a length as fractions of the text, and pieces.
    pub type RawEdit = (usize, usize, Vec<usize>);

    pub fn raw_edits() -> impl Strategy<Value = Vec<Vec<RawEdit>>> {
        let edit: BoxedStrategy<RawEdit> = (
            0usize..10_000,
            0usize..6,
            prop::collection::vec(0usize..PIECES.len(), 0..4),
        )
            .boxed();
        prop::collection::vec(prop::collection::vec(edit, 1..4), 1..6)
    }

    /// Turn a raw edit into an [`Edit`] that fits `model`, and apply it there.
    fn concrete(model: &mut Vec<char>, raw: &RawEdit) -> Edit {
        let len: usize = model.len();
        let from: usize = raw.0 % (len + 1);
        let to: usize = (from + raw.1).min(len);
        let text: String = raw.2.iter().map(|i: &usize| PIECES[*i]).collect();
        model.splice(from..to, text.chars());
        Edit {
            from: CharIdx::new(from),
            to: CharIdx::new(to),
            text,
        }
    }

    /// What the server says about the document now: tokens, then the format
    /// outcome. Roslyn's formatter can fail on some trivia (observed on
    /// U+0085 and U+2028); the same text must give the same failure.
    async fn witness(session: &Session, uri: &Value) -> (Value, Result<Value, String>) {
        let tokens: Value = session
            .raw(
                "textDocument/semanticTokens/full",
                json!({ "textDocument": { "uri": uri } }),
                DEADLINE,
            )
            .await
            .unwrap();
        let options: Value = json!({ "tabSize": 4, "insertSpaces": true });
        let format: Result<Value, String> = session
            .raw(
                "textDocument/formatting",
                json!({ "textDocument": { "uri": uri }, "options": options }),
                DEADLINE,
            )
            .await
            .map_err(|e: bb_intel::IntelError| e.to_string());
        (tokens["data"].clone(), format)
    }

    pub async fn one_case(
        session: &mut Session,
        doc: &DocPath,
        base: &str,
        batches: &[Vec<RawEdit>],
    ) -> Result<(), TestCaseError> {
        if session.version(doc).is_some() {
            session.close(doc).await.unwrap();
        }
        session.open_text(doc, base).await.unwrap();
        let uri: Value = json!(url::Url::from_file_path(doc.as_path()).unwrap());
        let mut model: Vec<char> = base.chars().collect();
        let mut last: DocVersion = session.version(doc).unwrap();
        for batch in batches {
            let edits: Vec<Edit> = batch
                .iter()
                .map(|r: &RawEdit| concrete(&mut model, r))
                .collect();
            let version: DocVersion = session.change(doc, &edits).await.unwrap();
            prop_assert!(version > last);
            last = version;
        }
        let mirror: String = session.text(doc).unwrap();
        prop_assert_eq!(&mirror, &model.iter().collect::<String>());
        let incremental: (Value, Result<Value, String>) = witness(session, &uri).await;
        session.resync(doc).await.unwrap();
        let full: (Value, Result<Value, String>) = witness(session, &uri).await;
        session.close(doc).await.unwrap();
        prop_assert!(incremental
            .0
            .as_array()
            .is_some_and(|a: &Vec<Value>| !a.is_empty()));
        prop_assert_eq!(incremental, full);
        Ok(())
    }
}

#[test]
fn random_edit_sequences_keep_the_server_text_equal_to_the_mirror() {
    let runtime: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
    let harness: Harness = runtime.block_on(start("props"));
    let doc: DocPath = harness.doc("_NUnitApex/GreeterSpec.cs");
    let base: String = std::fs::read_to_string(doc.as_path()).unwrap();
    let cell: RefCell<Harness> = RefCell::new(harness);
    let mut runner: TestRunner = TestRunner::new(Config {
        cases: 24,
        failure_persistence: None,
        ..Config::default()
    });
    let outcome: Result<(), proptest::test_runner::TestError<Vec<Vec<RawEdit>>>> =
        runner.run(&raw_edits(), |batches: Vec<Vec<RawEdit>>| {
            let mut h: std::cell::RefMut<'_, Harness> = cell.borrow_mut();
            runtime.block_on(one_case(&mut h.session, &doc, &base, &batches))
        });
    runtime.block_on(cell.into_inner().finish());
    outcome.unwrap();
}

#[test]
fn an_edit_starting_inside_a_crlf_pair_keeps_the_server_text_equal() {
    let runtime: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
    let mut harness: Harness = runtime.block_on(start("crlf"));
    let doc: DocPath = harness.doc("_NUnitApex/GreeterSpec.cs");
    let base: String = std::fs::read_to_string(doc.as_path()).unwrap();
    let batches: Vec<Vec<RawEdit>> = vec![
        vec![(0, 2, vec![1, 2, 5]), (0, 0, vec![2])],
        vec![(0, 2, vec![]), (2434, 0, vec![4, 2, 2])],
        vec![
            (8327, 2, vec![1, 9, 9]),
            (6425, 1, vec![10, 3]),
            (7741, 5, vec![0, 12]),
        ],
    ];
    let outcome: Result<(), proptest::test_runner::TestCaseError> =
        runtime.block_on(one_case(&mut harness.session, &doc, &base, &batches));
    runtime.block_on(harness.finish());
    outcome.unwrap();
}
