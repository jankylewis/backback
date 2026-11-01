use std::path::{Path, PathBuf};

use bb_intel::{Diagnostic, DocVersion, Severity};

use super::{Diags, Sent};
use crate::intel::model::tests::diag;

fn v(n: i32) -> DocVersion {
    DocVersion::from_wire(n).unwrap()
}

fn a() -> PathBuf {
    PathBuf::from("/ws/A.cs")
}

/// A tab of `/ws/A.cs` at version `at`.
fn at(at: i32) -> Diags {
    let mut diags: Diags = Diags::default();
    diags.sent(Sent::Open(a()));
    for _ in 1..at {
        diags.sent(Sent::Change(a()));
    }
    diags
}

#[test]
fn every_sync_request_moves_the_version_the_server_will_give() {
    let diags: Diags = at(3);
    assert_eq!(diags.shown(&a()), Some(v(3)));
    let mut reopened: Diags = at(3);
    reopened.sent(Sent::Open(a()));
    assert_eq!(reopened.shown(&a()), Some(DocVersion::first()));
}

#[test]
fn a_list_shows_only_on_the_version_it_was_computed_for() {
    let mut diags: Diags = at(1);
    assert!(diags.set(a(), v(1), vec![diag(1, Some(Severity::Error))]));
    assert_eq!(diags.of(&a()).len(), 1);
    // The user types: the old list stays out of sight, no squiggle sits on
    // text it was not computed for.
    diags.sent(Sent::Change(a()));
    assert_eq!(diags.of(&a()), Vec::<Diagnostic>::new().as_slice());
    assert_eq!(
        diags
            .all()
            .map(|(_, l): (&Path, &[Diagnostic])| l.len())
            .sum::<usize>(),
        0
    );
    assert!(diags.set(a(), v(2), vec![diag(2, None), diag(3, None)]));
    assert_eq!(diags.of(&a()).len(), 2);
}

#[test]
fn a_stale_report_never_replaces_a_newer_one() {
    let mut diags: Diags = at(3);
    assert!(diags.set(a(), v(3), vec![diag(1, Some(Severity::Error))]));
    assert!(!diags.set(a(), v(2), Vec::new()));
    assert_eq!(diags.of(&a()).len(), 1);
}

#[test]
fn a_report_for_a_tab_never_opened_or_closed_shows_nothing() {
    let mut diags: Diags = Diags::default();
    assert!(diags.set(a(), v(1), vec![diag(1, None)]));
    assert_eq!(diags.of(&a()).len(), 0);
    let mut closed: Diags = at(1);
    assert!(closed.set(a(), v(1), vec![diag(1, None)]));
    closed.closed(&a());
    assert_eq!(closed.of(&a()).len(), 0);
    let mut gone: Diags = at(2);
    assert!(gone.set(a(), v(2), vec![diag(1, None)]));
    gone.clear();
    assert_eq!(gone.of(&a()).len(), 0);
}
