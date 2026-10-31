use bb_explorer::{Entry, EntryKind};
use bb_ui::{theme::Theme, Rgb};
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
    svg()
        .data(data)
        .size(px(18.0))
        .flex_shrink_0()
        .text_color(tint)
}

pub(crate) fn tint(entry: &Entry, theme: &Theme) -> Rgb {
    if entry.dimmed() {
        return theme.text_dim;
    }
    match entry.kind() {
        EntryKind::Folder => theme.icon_folder,
        EntryKind::CSharp => theme.icon_csharp,
        EntryKind::Markdown => theme.icon_markdown,
        EntryKind::Project | EntryKind::Solution => theme.icon_project,
        EntryKind::Other => theme.icon_file,
    }
}
