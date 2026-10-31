use bb_explorer::{Entry, EntryKind};

pub(crate) fn chevron(entry: &Entry) -> &'static str {
    if entry.expandable() {
        if entry.expanded() {
            "⌄"
        } else {
            "›"
        }
    } else {
        " "
    }
}

pub(crate) fn icon(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Folder => "▣",
        EntryKind::CSharp => "C#",
        EntryKind::Markdown => "M↓",
        EntryKind::Project => "◇",
        EntryKind::Solution => "◈",
        EntryKind::Other => "·",
    }
}
