//! A right-click on an Explorer row (BB-BUG-050, BB-BUG-052), as in VS
//! Code: New File..., New Folder..., Open Terminal Here, Copy, Paste, Copy
//! Path, Rename... and Delete (to the Trash). A name is typed in the Go to
//! File box; a delete asks first. A file with unsaved changes is never
//! renamed or deleted under its tab; `--no-write` refuses every change.

use std::path::{Path, PathBuf};

use gpui::{Context, Pixels, Point, Window};

pub(crate) mod bb_ops;
mod jobs;
#[cfg(test)]
mod tests;
mod view;
pub(crate) use view::menu;

use crate::ExplorerView;

/// The Explorer's menu state: the open menu, and what Copy last took.
#[derive(Default)]
pub(crate) struct Menus {
    pub(crate) open: Option<FileMenu>,
    pub(crate) copied: Option<PathBuf>,
}

/// The open menu: the row's path, whether it is the root, where the click was.
pub(crate) struct FileMenu {
    pub(crate) path: PathBuf,
    pub(crate) root: bool,
    pub(crate) at: Point<Pixels>,
}

/// What a typed name is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NameOp {
    NewFile,
    NewFolder,
    Rename,
}

impl NameOp {
    pub(crate) fn title(self) -> &'static str {
        match self {
            NameOp::NewFile => "New File",
            NameOp::NewFolder => "New Folder",
            NameOp::Rename => "Rename",
        }
    }
}

/// A name box's job: the op and the folder it adds to, or the entry it renames.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NameAsk {
    pub(crate) op: NameOp,
    pub(crate) target: PathBuf,
}

/// One menu row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Item {
    Name(NameOp),
    Terminal,
    Copy,
    Paste,
    CopyPath,
    Delete,
}

impl Item {
    /// Whether the row changes the folder (and so `--no-write` refuses it).
    fn writes(self) -> bool {
        match self {
            Item::Name(_) | Item::Paste | Item::Delete => true,
            Item::Terminal | Item::Copy | Item::CopyPath => false,
        }
    }
}

/// A right-click on the row of `path`.
pub(crate) fn open_menu(view: &mut ExplorerView, path: PathBuf, at: Point<Pixels>) {
    let root: bool = path == view.tree.root().path();
    view.file_menu.open = Some(FileMenu { path, root, at });
}

/// A menu row's click: the name box, the delete question, a copy or paste,
/// the path to the clipboard, or the terminal in the row's folder.
pub(crate) fn apply(
    view: &mut ExplorerView,
    item: Item,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(menu): Option<FileMenu> = view.file_menu.open.take() else {
        return;
    };
    if item.writes() && view.tabs.no_write() {
        view.report(&"File change refused: --no-write keeps the folder unchanged");
        return;
    }
    match item {
        Item::Name(op) => ask_name(view, op, &menu.path, window, cx),
        Item::Delete => jobs::ask_delete(menu.path, window, cx),
        Item::Terminal => crate::terminal::open_in(view, &folder_of(&menu.path), window, cx),
        Item::Copy => {
            view.status = Some(format!("Copied {}", file_name(&menu.path)));
            view.file_menu.copied = Some(menu.path);
        }
        Item::Paste => jobs::paste(view, folder_of(&menu.path), cx),
        Item::CopyPath => {
            let text: String = menu.path.to_string_lossy().into_owned();
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
            view.status = Some(format!("Copied the path of {}", file_name(&menu.path)));
        }
    }
}

/// The name box for a new entry in `path`'s folder, or a rename of `path`.
fn ask_name(
    view: &mut ExplorerView,
    op: NameOp,
    path: &Path,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let (target, typed): (PathBuf, String) = match op {
        NameOp::Rename => (path.to_path_buf(), file_name(path)),
        NameOp::NewFile | NameOp::NewFolder => (folder_of(path), String::new()),
    };
    view.quick_open = Some(crate::quick_open::QuickOpen::name_box(
        NameAsk { op, target },
        typed,
    ));
    window.focus(&view.quick_focus, cx);
}

/// Enter in the name box: make or rename, then show the result.
pub(crate) fn apply_name(
    view: &mut ExplorerView,
    ask: &NameAsk,
    typed: &str,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let done: Result<PathBuf, String> = match ask.op {
        NameOp::NewFile => {
            bb_ops::new_file(&ask.target, typed).map_err(|e: bb_ops::FileOpError| e.to_string())
        }
        NameOp::NewFolder => {
            bb_ops::new_folder(&ask.target, typed).map_err(|e: bb_ops::FileOpError| e.to_string())
        }
        NameOp::Rename => release(view, &ask.target).and_then(|closed: Vec<PathBuf>| {
            let to: PathBuf = bb_ops::rename(&ask.target, typed)
                .map_err(|e: bb_ops::FileOpError| e.to_string())?;
            reopen(view, (&ask.target, &to), &closed, window, cx);
            Ok(to)
        }),
    };
    match done {
        Ok(path) => {
            show(view, &path, cx);
            if ask.op == NameOp::NewFile {
                view.open_file(&path, window, cx);
            }
            view.status = Some(format!("{}: {}", ask.op.title(), file_name(&path)));
        }
        Err(why) => view.report(&why),
    }
}

/// The folder a new entry goes in: a folder row's own, else the file's.
fn folder_of(path: &Path) -> PathBuf {
    if path.is_dir() {
        return path.to_path_buf();
    }
    path.parent()
        .map_or_else(|| path.to_path_buf(), Path::to_path_buf)
}

pub(super) fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Close the tabs of `path` and below; refused while one has unsaved changes.
pub(super) fn release(view: &mut ExplorerView, path: &Path) -> Result<Vec<PathBuf>, String> {
    let under: Vec<(usize, PathBuf, bool)> = view
        .tabs
        .items()
        .iter()
        .enumerate()
        .filter(|(_, tab): &(usize, &crate::editor::Tab)| tab.path.starts_with(path))
        .map(|(i, tab): (usize, &crate::editor::Tab)| {
            (i, tab.path.clone(), tab.document.is_dirty())
        })
        .collect();
    if let Some((_, dirty, _)) = under.iter().find(|(_, _, d): &&(usize, PathBuf, bool)| *d) {
        return Err(format!(
            "Save or close {} first: it has unsaved changes",
            file_name(dirty)
        ));
    }
    for (index, _, _) in under.iter().rev() {
        let _closed: bool = view.tabs.close(*index);
    }
    Ok(under
        .into_iter()
        .map(|(_, p, _): (usize, PathBuf, bool)| p)
        .collect())
}

/// Open again, at their new place, the tabs a rename of `from` closed.
fn reopen(
    view: &mut ExplorerView,
    (from, to): (&Path, &Path),
    closed: &[PathBuf],
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    for old in closed {
        if let Ok(rest) = old.strip_prefix(from) {
            view.open_file(&to.join(rest), window, cx);
        }
    }
}

/// Take the change into the tree and select `path`'s row.
pub(super) fn show(view: &mut ExplorerView, path: &Path, cx: &mut Context<ExplorerView>) {
    let _changed: bool = view.tree.refresh().unwrap_or(false);
    if let Ok(Some(row)) = view.tree.reveal(path) {
        view.select_row(row, cx);
    }
}
