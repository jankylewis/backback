//! The gate: backback must pass its own audit. `cargo test` fails on any violation.

use std::path::{Path, PathBuf};

#[test]
fn backback_passes_its_own_conventions() {
    let crate_dir: &Path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo: PathBuf = crate_dir.join("../../..");
    let found: Vec<bb_audit::Violation> = bb_audit::audit(&repo).unwrap();
    let shown: Vec<String> = found
        .iter()
        .map(|v: &bb_audit::Violation| v.to_string())
        .collect();
    assert!(
        found.is_empty(),
        "{} violations:\n{}",
        found.len(),
        shown.join("\n")
    );
}
