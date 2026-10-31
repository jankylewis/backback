use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

/// Explorer entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// Directory.
    Folder,
    /// C# source.
    CSharp,
    /// Markdown document.
    Markdown,
    /// MSBuild project.
    Project,
    /// Solution file.
    Solution,
    /// Other file or symlink.
    Other,
}

/// One tree entry, with children loaded only on expansion.
#[derive(Debug, Clone)]
pub struct Entry {
    pub(super) path: PathBuf,
    name: String,
    pub(super) kind: EntryKind,
    pub(super) dimmed: bool,
    pub(super) expanded: bool,
    pub(super) children: Option<Vec<Entry>>,
}

impl Entry {
    pub(super) fn new(path: PathBuf, kind: EntryKind, dimmed: bool) -> Self {
        let name: String = path.file_name().map_or_else(
            || path.display().to_string(),
            |value: &std::ffi::OsStr| value.to_string_lossy().into_owned(),
        );
        Self {
            path,
            name,
            kind,
            dimmed,
            expanded: false,
            children: None,
        }
    }

    /// Full path, suitable for a tooltip.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Entry kind for an icon.
    #[must_use]
    pub fn kind(&self) -> EntryKind {
        self.kind
    }
    /// Whether to de-emphasize generated or hidden content.
    #[must_use]
    pub fn dimmed(&self) -> bool {
        self.dimmed
    }
    /// Whether descendants are visible.
    #[must_use]
    pub fn expanded(&self) -> bool {
        self.expanded
    }
    /// Whether this entry can reveal children.
    #[must_use]
    pub fn expandable(&self) -> bool {
        self.kind == EntryKind::Folder || (self.kind == EntryKind::Solution && self.loaded())
    }
    /// Whether children have been read.
    #[must_use]
    pub fn loaded(&self) -> bool {
        self.children.is_some()
    }
    /// Already loaded children.
    #[must_use]
    pub fn children(&self) -> &[Entry] {
        self.children.as_deref().unwrap_or(&[])
    }
}
pub(super) fn file_kind(path: &Path) -> EntryKind {
    match path
        .extension()
        .map(|ext: &std::ffi::OsStr| ext.to_string_lossy().to_ascii_lowercase())
        .as_deref()
    {
        Some("cs") => EntryKind::CSharp,
        Some("md") => EntryKind::Markdown,
        Some("csproj" | "fsproj" | "vbproj") => EntryKind::Project,
        Some("slnx" | "sln") => EntryKind::Solution,
        Some(_) | None => EntryKind::Other,
    }
}

pub(super) fn dim_name(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "bin" | "obj" | ".git")
}

pub(super) fn sort_entries(entries: &mut [Entry]) {
    entries.sort_by(|left: &Entry, right: &Entry| {
        let folders: Ordering =
            matches!(right.kind, EntryKind::Folder).cmp(&matches!(left.kind, EntryKind::Folder));
        folders.then_with(|| name_order(&left.name, &right.name))
    });
}

pub(crate) fn name_order(left: &str, right: &str) -> Ordering {
    left.to_ascii_lowercase()
        .cmp(&right.to_ascii_lowercase())
        .then_with(|| left.cmp(right))
}

pub(super) fn find<'a>(entry: &'a Entry, path: &Path) -> Option<&'a Entry> {
    if entry.path == path {
        return Some(entry);
    }
    entry
        .children()
        .iter()
        .find_map(|child: &Entry| find(child, path))
}

pub(super) fn find_mut<'a>(entry: &'a mut Entry, path: &Path) -> Option<&'a mut Entry> {
    if entry.path == path {
        return Some(entry);
    }
    entry
        .children
        .as_mut()?
        .iter_mut()
        .find_map(|child: &mut Entry| find_mut(child, path))
}

pub(super) fn collect_visible<'a>(
    entry: &'a Entry,
    depth: usize,
    rows: &mut Vec<(usize, &'a Entry)>,
) {
    rows.push((depth, entry));
    if entry.expanded {
        for child in entry.children() {
            collect_visible(child, depth + 1, rows);
        }
    }
}
