//! Go to File (Cmd+Shift+O): every file under the root, filtered by a typed
//! query as a fuzzy subsequence; names that match rank first. Go to Symbol
//! (Cmd+F12): the active file's outline in the same box, in source order.

use std::path::{Path, PathBuf};

/// Folders never listed: VCS, build output, dependencies.
const SKIP: [&str; 5] = [".git", "bin", "obj", "node_modules", ".target"];
/// Enough for any solution; a huge folder stops listing here.
const MAX_FILES: usize = 50_000;
/// Rows shown at once.
pub(crate) const ROWS: usize = 14;

/// What Enter picked.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Chosen {
    /// A file to open.
    File(PathBuf),
    /// A 0-based line of the active file.
    Line(usize),
}

struct Item {
    label: PathBuf,
    line: usize,
}

pub(crate) struct QuickOpen {
    pub(crate) query: String,
    pub(crate) selected: usize,
    /// `Some` for Go to File (labels are paths under it), `None` for symbols.
    root: Option<PathBuf>,
    items: Vec<Item>,
}

impl QuickOpen {
    pub(crate) fn new(root: &Path) -> QuickOpen {
        let mut files: Vec<PathBuf> = Vec::new();
        walk(root, root, &mut files);
        files.sort();
        QuickOpen {
            query: String::new(),
            selected: 0,
            root: Some(root.to_path_buf()),
            items: files
                .into_iter()
                .map(|label: PathBuf| Item { label, line: 0 })
                .collect(),
        }
    }

    /// Go to Symbol over `outline`: (label, 0-based line), in source order.
    pub(crate) fn symbols(outline: Vec<(String, usize)>) -> QuickOpen {
        QuickOpen {
            query: String::new(),
            selected: 0,
            root: None,
            items: outline
                .into_iter()
                .map(|(label, line): (String, usize)| Item {
                    label: PathBuf::from(label),
                    line,
                })
                .collect(),
        }
    }

    /// The box's title.
    pub(crate) fn title(&self) -> &'static str {
        if self.root.is_some() {
            "Go to File"
        } else {
            "Go to Symbol"
        }
    }

    /// The best matches, relative to the root, best first.
    pub(crate) fn matches(&self) -> Vec<&Path> {
        let query: String = self.query.to_lowercase();
        let files: bool = self.root.is_some();
        let mut scored: Vec<(u8, usize, &Path)> = self
            .items
            .iter()
            .filter_map(|item: &Item| {
                let order: usize = if files {
                    item.label.as_os_str().len()
                } else {
                    item.line
                };
                score(&item.label, &query).map(|s: u8| (s, order, item.label.as_path()))
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

    /// The selected match: a file to open, or a line of the active file.
    pub(crate) fn chosen(&self) -> Option<Chosen> {
        let matches: Vec<&Path> = self.matches();
        let label: &Path = matches.get(self.selected)?;
        let item: &Item = self
            .items
            .iter()
            .find(|item: &&Item| std::ptr::eq(item.label.as_path(), label))?;
        Some(match &self.root {
            Some(root) => Chosen::File(root.join(&item.label)),
            None => Chosen::Line(item.line),
        })
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
            Some(Chosen::File(dir.path().join("Specs/GreeterSpec.cs")))
        );
        quick.type_text("zzz");
        assert_eq!(quick.chosen(), None);
        quick.backspace();
        quick.backspace();
        quick.backspace();
        assert_eq!(quick.matches().len(), 2);
    }

    #[test]
    fn symbols_keep_source_order_and_enter_gives_the_line() {
        let mut quick: QuickOpen = QuickOpen::symbols(vec![
            ("class Greeter".to_owned(), 2),
            ("  method Hello".to_owned(), 4),
            ("  method Help".to_owned(), 9),
        ]);
        assert_eq!(quick.title(), "Go to Symbol");
        assert_eq!(quick.matches().len(), 3);
        quick.type_text("hel");
        quick.step(true);
        assert_eq!(quick.chosen(), Some(Chosen::Line(9)));
    }
}
