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

#[cfg(test)]
mod links {
    use std::path::{Path, PathBuf};

    /// Folders that hold no repo Markdown.
    const SKIP: [&str; 3] = [".git", ".target", "target"];

    /// Every `.md` file below `dir`.
    pub fn markdown(dir: &Path) -> Vec<PathBuf> {
        let mut found: Vec<PathBuf> = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path: PathBuf = entry.unwrap().path();
            let name: String = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() && !SKIP.contains(&name.as_str()) {
                found.extend(markdown(&path));
            } else if path
                .extension()
                .is_some_and(|e: &std::ffi::OsStr| e.eq_ignore_ascii_case("md"))
            {
                found.push(path);
            }
        }
        found
    }

    /// Local targets of `[text](target)` links, without any `#anchor`.
    pub fn local_targets(text: &str) -> Vec<String> {
        text.split("](")
            .skip(1)
            .filter_map(|rest: &str| rest.split_once(')'))
            .map(|(target, _): (&str, &str)| target.split('#').next().unwrap().to_owned())
            .filter(|t: &String| !t.is_empty() && !t.contains("://") && !t.starts_with("mailto:"))
            .collect()
    }
}

#[test]
fn every_local_markdown_link_resolves() {
    let repo: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut broken: Vec<String> = Vec::new();
    for file in links::markdown(&repo) {
        let text: String = std::fs::read_to_string(&file).unwrap();
        for target in links::local_targets(&text) {
            if !file.parent().unwrap().join(&target).exists() {
                let rel: &Path = file.strip_prefix(&repo).unwrap();
                broken.push(format!("{} -> {target}", rel.display()));
            }
        }
    }
    assert!(broken.is_empty(), "broken links:\n{}", broken.join("\n"));
}
