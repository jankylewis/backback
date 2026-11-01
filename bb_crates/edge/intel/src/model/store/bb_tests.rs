use proptest::prelude::*;

use super::{accepts, Entry, Map, Recorded, Store};
use crate::model::bb_doc::DocVersion;
use crate::model::DocPath;
use crate::IntelError;

fn at(n: i32) -> DocVersion {
    DocVersion::from_wire(n).unwrap()
}

fn path() -> DocPath {
    DocPath::new("/ws/A.cs".into()).unwrap()
}

fn entry(version: Option<i32>, codes: &[&str]) -> Entry {
    let list: Vec<lsp_types::Diagnostic> = codes
        .iter()
        .map(|code: &&str| lsp_types::Diagnostic {
            message: (*code).to_owned(),
            ..lsp_types::Diagnostic::default()
        })
        .collect();
    Entry {
        version: version.map(at),
        list,
    }
}

#[test]
fn a_stale_report_never_replaces_a_newer_one() {
    let store: Store = Store::default();
    assert_eq!(
        store.record(&path(), entry(Some(3), &["new"])).unwrap(),
        Recorded::Kept
    );
    assert_eq!(
        store.record(&path(), entry(Some(2), &[])).unwrap(),
        Recorded::Stale
    );
    assert_eq!(
        store.record(&path(), entry(None, &[])).unwrap(),
        Recorded::Stale
    );
    let kept: Entry = store.entry(&path()).unwrap().unwrap();
    assert_eq!(kept.version, Some(at(3)));
    assert_eq!(kept.list.len(), 1);
}

#[test]
fn a_same_or_newer_report_replaces_the_stored_one() {
    let store: Store = Store::default();
    let _push: Recorded = store.record(&path(), entry(None, &["push"])).unwrap();
    assert_eq!(
        store.record(&path(), entry(Some(1), &["a"])).unwrap(),
        Recorded::Kept
    );
    assert_eq!(
        store.record(&path(), entry(Some(1), &["b"])).unwrap(),
        Recorded::Kept
    );
    assert_eq!(
        store.record(&path(), entry(Some(4), &[])).unwrap(),
        Recorded::Kept
    );
    assert_eq!(store.entry(&path()).unwrap().unwrap().version, Some(at(4)));
}

#[test]
fn a_poisoned_store_is_an_error_not_an_empty_list() {
    let store: Store = Store::default();
    let _first: Recorded = store.record(&path(), entry(Some(1), &["x"])).unwrap();
    let shared: Store = store.clone();
    let poisoned: std::thread::Result<()> = std::thread::spawn(move || {
        let _held: std::sync::MutexGuard<'_, Map> = shared.0.lock().unwrap();
        panic!("poison the store");
    })
    .join();
    assert!(poisoned.is_err());
    assert!(matches!(
        store.entry(&path()),
        Err(IntelError::StorePoisoned)
    ));
    assert!(matches!(
        store.record(&path(), entry(Some(2), &[])),
        Err(IntelError::StorePoisoned)
    ));
    assert!(matches!(
        store.remove(&path()),
        Err(IntelError::StorePoisoned)
    ));
}

#[test]
fn a_wire_version_below_one_is_refused() {
    assert_eq!(DocVersion::from_wire(0), None);
    assert_eq!(DocVersion::from_wire(-1), None);
    assert_eq!(DocVersion::from_wire(1), Some(DocVersion::first()));
}

proptest! {
    /// Whatever order reports arrive in, the stored one has the highest
    /// version seen, and an unversioned report never displaces a versioned one.
    #[test]
    fn the_store_keeps_the_newest_version(order in proptest::collection::vec(
        proptest::option::of(1_i32..50), 1..40)) {
        let store: Store = Store::default();
        for version in &order {
            let _each: Recorded = store.record(&path(), entry(*version, &[])).unwrap();
        }
        let newest: Option<DocVersion> = order.iter().flatten().max().copied().map(at);
        let stored: Option<DocVersion> = store.entry(&path()).unwrap().unwrap().version;
        prop_assert_eq!(stored, newest);
    }

    #[test]
    fn accepts_is_monotone(stored in proptest::option::of(1_i32..50),
                           incoming in proptest::option::of(1_i32..50)) {
        let want: bool = match (stored, incoming) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(s), Some(i)) => i >= s,
        };
        prop_assert_eq!(accepts(stored.map(at), incoming.map(at)), want);
    }
}
