//! Go to File (Cmd+Shift+O): every file under the root, filtered by a typed
//! query as a fuzzy subsequence; names that match rank first.

use std::path::{Path, PathBuf};

/// Folders never listed: VCS, build output, dependencies.
const SKIP: [&str; 5] = [".git", "bin", "obj", "node_modules", ".target"];
/// Enough for any solution; a huge folder stops listing here.
const MAX_FILES: usize = 50_000;
/// Rows shown at once.
pub(crate) const ROWS: usize = 14;

pub(crate) struct QuickOpen {
    pub(crate) query: String,
    pub(crate) selected: usize,
    root: PathBuf,
    files: Vec<PathBuf>,
}

impl QuickOpen {
    pub(crate) fn new(root: &Path) -> QuickOpen {
        let mut files: Vec<PathBuf> = Vec::new();
        walk(root, root, &mut files);
        files.sort();
        QuickOpen {
            query: String::new(),
            selected: 0,
            root: root.to_path_buf(),
            files,
        }
    }

    /// The best matches, relative to the root, best first.
    pub(crate) fn matches(&self) -> Vec<&Path> {
        let query: String = self.query.to_lowercase();
        let mut scored: Vec<(u8, usize, &Path)> = self
            .files
            .iter()
            .filter_map(|file: &PathBuf| {
                score(file, &query).map(|s: u8| (s, file.as_os_str().len(), file.as_path()))
            })
            .collect();
        scored.sort_by(|a: &(u8, usize, &Path), b: &(u8, usize, &Path)| {
            a.0.cmp(&b.0).then(a.1.cmp(&b.1))
        });
        scored
            .into_iter()
            .take(ROWS)
            .map(|(_, _, path): (u8, usize, &Path)| path)
            .collect()
    }

    /// The absolute path of the selected match.
    pub(crate) fn chosen(&self) -> Option<PathBuf> {
        let matches: Vec<&Path> = self.matches();
        matches
            .get(self.selected)
            .map(|path: &&Path| self.root.join(path))
    }

    pub(crate) fn type_text(&mut self, text: &str) {
        self.query.push_str(text);
        self.selected = 0;
    }

    pub(crate) fn backspace(&mut self) {
        self.query.pop();
        self.selected = 0;
    }

    pub(crate) fn step(&mut self, down: bool) {
        let last: usize = self.matches().len().saturating_sub(1);
        self.selected = if down {
            self.selected.saturating_add(1).min(last)
        } else {
            self.selected.saturating_sub(1)
        };
    }
}

/// 0: the query is in the file name; 1: in the path; None: no match.
fn score(file: &Path, query: &str) -> Option<u8> {
    let name: String = file.file_name()?.to_string_lossy().to_lowercase();
    if subsequence(&name, query) {
        return Some(0);
    }
    subsequence(&file.to_string_lossy().to_lowercase(), query).then_some(1)
}

fn subsequence(text: &str, query: &str) -> bool {
    let mut chars: std::str::Chars<'_> = text.chars();
    query
        .chars()
        .all(|wanted: char| chars.any(|ch: char| ch == wanted))
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(items): Result<std::fs::ReadDir, std::io::Error> = std::fs::read_dir(dir) else {
        return;
    };
    for item in items.flatten() {
        if out.len() >= MAX_FILES {
            return;
        }
        let path: PathBuf = item.path();
        let name: String = item.file_name().to_string_lossy().into_owned();
        if SKIP.contains(&name.as_str()) || name.starts_with('.') && path.is_dir() {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, out);
        } else if let Ok(relative) = path.strip_prefix(root) {
            out.push(relative.to_path_buf());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> (tempfile::TempDir, QuickOpen) {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        for file in [
            "Core/Greeter.cs",
            "Core/obj/Gen.cs",
            "Specs/GreeterSpec.cs",
            "README.md",
            ".git/HEAD",
        ] {
            let path: PathBuf = dir.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        }
        let quick: QuickOpen = QuickOpen::new(dir.path());
        (dir, quick)
    }

    #[test]
    fn build_output_and_git_are_never_listed() {
        let (_dir, quick): (tempfile::TempDir, QuickOpen) = tree();
        let all: Vec<&Path> = quick.matches();
        assert_eq!(all.len(), 3, "{all:?}");
        assert!(all
            .iter()
            .all(|p: &&Path| !p.starts_with(".git") && !p.to_string_lossy().contains("obj")));
    }

    #[test]
    fn a_fuzzy_query_ranks_name_matches_first_and_enter_gives_the_path() {
        let (dir, mut quick): (tempfile::TempDir, QuickOpen) = tree();
        quick.type_text("grt");
        let names: Vec<String> = quick
            .matches()
            .iter()
            .map(|p: &&Path| p.display().to_string())
            .collect();
        assert_eq!(names, ["Core/Greeter.cs", "Specs/GreeterSpec.cs"]);
        quick.step(true);
        assert_eq!(
            quick.chosen(),
            Some(dir.path().join("Specs/GreeterSpec.cs"))
        );
        quick.type_text("zzz");
        assert_eq!(quick.chosen(), None);
        quick.backspace();
        quick.backspace();
        quick.backspace();
        assert_eq!(quick.matches().len(), 2);
    }
}
