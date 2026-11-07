//! Go to File (Cmd+Shift+O): every file under the root, filtered by a typed
//! query as a fuzzy subsequence; names that match rank first. Go to Symbol
//! (Cmd+F12): the active file's outline in the same box, in source order.

mod bb_line;
mod places;
mod rank;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

pub(crate) use bb_line::split_at;
pub(crate) use places::{Field, FindAsk, FindFields};

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
    /// A Find in Files query, where and which files.
    Find(FindAsk),
    /// A typed name for a new entry or a rename (BB-BUG-050).
    Name(crate::files::NameAsk, String),
}

struct Item {
    label: PathBuf,
    line: usize,
}

pub(crate) struct QuickOpen {
    pub(crate) query: String,
    /// The caret in `query`: a byte offset on a char boundary.
    pub(crate) caret: usize,
    pub(crate) selected: usize,
    /// `Some` for Go to File (labels are paths under it), `None` for symbols.
    root: Option<PathBuf>,
    items: Vec<Item>,
    /// Find in Files: its fields; the list is the places to search.
    find: Option<FindFields>,
    /// Go to File before its background walk has filled the list.
    loading: bool,
    /// A name box for an Explorer change: no list, Enter gives the name.
    name: Option<crate::files::NameAsk>,
}

impl QuickOpen {
    /// Go to File over `root`, listing files on the calling thread (tests).
    #[cfg(test)]
    pub(crate) fn new(root: &Path) -> QuickOpen {
        let mut quick: QuickOpen = QuickOpen::listing(root);
        quick.fill(QuickOpen::list_files(root));
        quick
    }

    /// Go to File over `root` with no list yet: `fill` adds it after the
    /// walk (`list_files`) has run off the UI thread (BB-PRF-002).
    pub(crate) fn listing(root: &Path) -> QuickOpen {
        QuickOpen {
            root: Some(root.to_path_buf()),
            loading: true,
            ..QuickOpen::blank()
        }
    }

    /// An empty box: no list, no title of its own.
    fn blank() -> QuickOpen {
        QuickOpen {
            query: String::new(),
            caret: 0,
            selected: 0,
            root: None,
            items: Vec::new(),
            find: None,
            loading: false,
            name: None,
        }
    }

    /// Every file under `root`, relative to it and sorted; may be slow.
    pub(crate) fn list_files(root: &Path) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = bb_search::files(root)
            .take(MAX_FILES)
            .filter_map(|path: PathBuf| path.strip_prefix(root).ok().map(Path::to_path_buf))
            .collect();
        files.sort();
        files
    }

    /// The walk's result for the box that waits for `root`'s files.
    pub(crate) fn fill(&mut self, files: Vec<PathBuf>) {
        self.items = files
            .into_iter()
            .map(|label: PathBuf| Item { label, line: 0 })
            .collect();
        self.loading = false;
    }

    /// True while this box waits for a walk of `root`.
    pub(crate) fn waits_for(&self, root: &Path) -> bool {
        self.loading && self.root.as_deref() == Some(root)
    }

    /// Go to Symbol over `outline`: (label, 0-based line), in source order.
    pub(crate) fn symbols(outline: Vec<(String, usize)>) -> QuickOpen {
        QuickOpen {
            items: outline
                .into_iter()
                .map(|(label, line): (String, usize)| Item {
                    label: PathBuf::from(label),
                    line,
                })
                .collect(),
            ..QuickOpen::blank()
        }
    }

    /// Find in Files' fields, for the view.
    pub(crate) fn fields(&self) -> Option<&FindFields> {
        self.find.as_ref()
    }

    /// A name box for `ask`, holding `typed` (a rename starts with the name).
    pub(crate) fn name_box(ask: crate::files::NameAsk, typed: String) -> QuickOpen {
        QuickOpen {
            caret: typed.len(),
            query: typed,
            name: Some(ask),
            ..QuickOpen::blank()
        }
    }

    /// The box's title.
    /// One readback line: title, query, listed items, matches, loading.
    pub(crate) fn describe(&self) -> String {
        format!(
            "bb_quick_state title={:?} query={:?} items={} matches={} loading={}",
            self.title(),
            self.query,
            self.items.len(),
            self.matches().len(),
            self.loading
        )
    }

    pub(crate) fn title(&self) -> &'static str {
        if let Some(ask) = &self.name {
            ask.op.title()
        } else if self.find.is_some() {
            "Find in Files"
        } else if self.loading {
            "Go to File (listing files...)"
        } else if self.root.is_some() {
            "Go to File"
        } else {
            "Go to Symbol"
        }
    }

    /// The best matches, relative to the root, best first.
    pub(crate) fn matches(&self) -> Vec<&Path> {
        let query: String = self
            .find
            .as_ref()
            .map_or(&self.query, |f: &FindFields| &f.in_text)
            .to_lowercase();
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
                rank::score(&item.label, &query).map(|s: u8| (s, order, item.label.as_path()))
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
        if let Some(ask) = &self.name {
            let typed: Option<String> =
                Some(self.query.trim().to_owned()).filter(|t: &String| !t.is_empty());
            return typed.map(|t: String| Chosen::Name(ask.clone(), t));
        }
        if let Some(fields) = &self.find {
            return self.find_ask(fields).map(Chosen::Find);
        }
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

    pub(crate) fn step(&mut self, down: bool) {
        let last: usize = self.matches().len().saturating_sub(1);
        self.selected = if down {
            self.selected.saturating_add(1).min(last)
        } else {
            self.selected.saturating_sub(1)
        };
    }
}
