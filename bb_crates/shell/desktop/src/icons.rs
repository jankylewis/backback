use bb_explorer::{Entry, EntryKind};
use gpui::{prelude::*, px, svg, Rgba, Svg};

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

pub(crate) fn icon(entry: &Entry, tint: Rgba) -> Svg {
    let data: &'static [u8] = match entry.kind() {
        EntryKind::Folder if entry.expanded() => include_bytes!("../assets/bb_folders/open.svg"),
        EntryKind::Folder => include_bytes!("../assets/bb_folders/bb_closed.svg"),
        EntryKind::CSharp => include_bytes!("../assets/files/bb_csharp.svg"),
        EntryKind::Markdown => include_bytes!("../assets/files/markdown.svg"),
        EntryKind::Project => include_bytes!("../assets/files/project.svg"),
        EntryKind::Solution => include_bytes!("../assets/files/solution.svg"),
        EntryKind::Other => include_bytes!("../assets/files/generic.svg"),
    };
    svg().data(data).size(px(18.0)).text_color(tint)
}
