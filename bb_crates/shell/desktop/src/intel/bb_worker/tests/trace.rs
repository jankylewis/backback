//! The GUI worker opens and queries a real solution without changing it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tokio::sync::mpsc::UnboundedReceiver;

use super::{first_diagnostics, harness, start, wait_ready};
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::Link;

fn snapshot(root: &Path) -> BTreeMap<PathBuf, (u64, SystemTime)> {
    let mut found: BTreeMap<PathBuf, (u64, SystemTime)> = BTreeMap::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path: PathBuf = entry.unwrap().path();
        let meta: std::fs::Metadata = path.metadata().unwrap();
        if path.is_dir() {
            found.extend(snapshot(&path));
            found.insert(path, (0, meta.modified().unwrap()));
        } else {
            found.insert(path, (meta.len(), meta.modified().unwrap()));
        }
    }
    found
}

#[tokio::test]
async fn gui_solution_open_and_document_sync_leave_no_trace() {
    let root: PathBuf = harness("gui-trace");
    let before: BTreeMap<PathBuf, (u64, SystemTime)> = snapshot(&root);
    let solution: PathBuf = root.join("Harness.slnx");
    let (link, mut events): (Link, UnboundedReceiver<Event>) = start(&solution);
    wait_ready(&mut events).await;
    let path: PathBuf = root.join("_NUnitApex/GreeterSpec.cs");
    let text: String = std::fs::read_to_string(&path).unwrap();
    assert!(link.send(Request::Open { path, text }));
    first_diagnostics(&mut events).await;
    assert_eq!(snapshot(&root), before);
}
