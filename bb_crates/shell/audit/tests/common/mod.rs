//! Throwaway trees for audit tests. Each test binary uses every helper here.

use std::path::PathBuf;

use bb_audit::{audit, Rule, Violation};

/// A temp folder that removes itself on drop.
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(tag: &str) -> Self {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("bb-audit-{tag}-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    /// Write `text` to `rel`, creating parent folders.
    pub fn file(&self, rel: &str, text: &str) {
        let path: PathBuf = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// Rename `from` to `to`, both relative to the root.
    pub fn rename(&self, from: &str, to: &str) {
        std::fs::rename(self.0.join(from), self.0.join(to)).unwrap();
    }

    /// Every violation as (relative path, rule), sorted.
    pub fn rules(&self) -> Vec<(String, Rule)> {
        let found: Vec<Violation> = audit(&self.0).unwrap();
        found
            .into_iter()
            .map(|v: Violation| (v.path.display().to_string(), v.rule))
            .collect()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _gone: std::io::Result<()> = std::fs::remove_dir_all(&self.0);
    }
}

/// The one violation expected at `path`.
pub fn only(path: &str, rule: Rule) -> Vec<(String, Rule)> {
    vec![(path.to_owned(), rule)]
}
