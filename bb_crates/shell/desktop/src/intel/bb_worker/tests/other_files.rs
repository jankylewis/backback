//! BB-BUG-019: an edit in one file changes the diagnostics of another open
//! file. Renaming `Greeter.Hello` breaks the call in the spec; the spec's
//! list must follow without an edit of its own, and clear again on undo.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_intel::{Diagnostic, Edit, Severity};
use bb_text::CharIdx;

use super::{first_diagnostics, loaded, Live};
use crate::intel::bb_worker::protocol::{Event, Request};

const GREETER: &str = "Core/Greeter.cs";

/// Whether `list` holds the spec's error for the missing `Hello`.
fn misses_hello(list: &[Diagnostic]) -> bool {
    list.iter()
        .any(|d: &Diagnostic| d.severity == Some(Severity::Error) && d.message.contains("'Hello'"))
}

/// Swap the five chars at `at` in Greeter.cs for `text`.
fn rename(live: &Live, greeter: &Path, at: usize, text: &str) {
    let from: CharIdx = CharIdx::new(at);
    let to: CharIdx = CharIdx::new(at + 5);
    assert!(live.link.send(Request::Change {
        path: greeter.to_path_buf(),
        edits: vec![Edit {
            from,
            to,
            text: text.to_owned(),
        }],
    }));
}

/// Wait, at most 90 s, for a diagnostics list of the spec where `wanted`
/// holds; the lists of other files are skipped.
async fn spec_list_where(live: &mut Live, wanted: fn(&[Diagnostic]) -> bool, what: &str) {
    let started: Instant = Instant::now();
    loop {
        let left: Duration = Duration::from_secs(90).saturating_sub(started.elapsed());
        let event: Option<Event> = tokio::time::timeout(left, live.events.recv())
            .await
            .unwrap_or_else(|_: tokio::time::error::Elapsed| {
                panic!("the spec never got a list where {what} within 90 s")
            });
        let Some(event): Option<Event> = event else {
            panic!("the server thread stopped");
        };
        if let Event::Diagnostics { path, list, .. } = event {
            if path == live.spec && wanted(&list) {
                return;
            }
        }
    }
}

#[tokio::test]
async fn an_edit_in_one_file_refreshes_the_diagnostics_of_the_other_open_files() {
    let mut live: Live = loaded("other-files").await;
    let greeter: PathBuf = live.root.join(GREETER);
    let text: String = std::fs::read_to_string(&greeter).unwrap();
    assert!(text.is_ascii(), "char offsets below assume ASCII");
    let at: usize = text.find("Hello").unwrap();
    assert!(live.link.send(Request::Open {
        path: greeter.clone(),
        text,
    }));
    first_diagnostics(&mut live.events).await;
    rename(&live, &greeter, at, "Howdy");
    spec_list_where(
        &mut live,
        misses_hello,
        "the call to the renamed Hello fails",
    )
    .await;
    rename(&live, &greeter, at, "Hello");
    spec_list_where(
        &mut live,
        |l: &[Diagnostic]| !misses_hello(l),
        "the error cleared",
    )
    .await;
}
