//! Cmd+N, as VS Code: a "New file" tab with no file on disk yet. Its first
//! save (Cmd+S, or Save when it closes, the window closes or the app quits)
//! asks where to store it, starting in the folder of the file last focused.

use std::path::{Path, PathBuf};

use bb_syntax::{CSharpSyntax, LiveTree, Tree};
use bb_text::Document;
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

use super::{Tab, TabError, TabEvent};
use crate::editor::Tabs;
use crate::ExplorerView;

/// The name a new file shows; the second open one is "New file 2".
const NEW_FILE: &str = "New file";

/// Open a new file (Cmd+N).
#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct NewFile;

impl Tab {
    /// An empty tab whose (not yet real) path is `dir/name`.
    pub(crate) fn untitled(dir: &Path, name: &str) -> Result<Tab, TabError> {
        let document: Document = Document::from_text("", None);
        let mut syntax: CSharpSyntax = CSharpSyntax::new()?;
        let tree: Tree = syntax.parse_rope_in(document.rope(), None, None)?;
        Ok(Tab {
            path: dir.join(name),
            document,
            highlighter: CSharpSyntax::new()?,
            live: LiveTree::new(syntax, tree),
            test_methods: Vec::new(),
            window_lines: None,
            read_only: false,
            scroll_x: 0.0,
            sighted: None,
            widest_px: 0.0,
            pinned: false,
            group: super::super::Group(0),
            wrap: None,
            braces: std::cell::Cell::new(None),
        })
    }

    /// True for a new file never saved: it has no file on disk.
    pub(crate) fn is_untitled(&self) -> bool {
        self.document.path().is_none()
    }
}

impl Tabs {
    /// Open a new file in `dir`, the active tab; the language server is not
    /// told of it until it is saved.
    pub(crate) fn new_file(&mut self, dir: &Path) -> Result<(), TabError> {
        // One more name than open tabs: one of them is always free.
        let name: String = (1..=self.items.len().saturating_add(1))
            .map(|n: usize| match n {
                1 => NEW_FILE.to_owned(),
                _ => format!("{NEW_FILE} {n}"),
            })
            .find(|name: &String| !self.untitled_named(name))
            .unwrap_or_else(|| NEW_FILE.to_owned());
        let tab: Tab = Tab::untitled(dir, &name)?;
        let path: PathBuf = tab.path.clone();
        let at: usize = self.admit(tab);
        self.events
            .retain(|event: &TabEvent| *event != TabEvent::Opened(path.clone()));
        self.switch(at);
        Ok(())
    }

    /// Whether a new file named `name` is open.
    fn untitled_named(&self, name: &str) -> bool {
        self.items
            .iter()
            .any(|tab: &Tab| tab.is_untitled() && tab.path.file_name() == Some(name.as_ref()))
    }

    /// Give the new file at `old` its saved `path`, as if it had been opened
    /// there.
    fn saved_as(&mut self, old: &Path, path: PathBuf) {
        if let Some(tab) = self.tab_mut(old) {
            tab.path.clone_from(&path);
        }
        if self.pending_close.as_deref() == Some(old) {
            self.pending_close = Some(path.clone());
        }
        self.events.push(TabEvent::Opened(path));
    }
}

/// Cmd+N: the new file starts in the folder of the file last focused, else
/// the workspace.
pub(crate) fn on_new_file(
    view: &mut ExplorerView,
    _: &NewFile,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let dir: PathBuf = view
        .tabs
        .active_tab()
        .and_then(|tab: &Tab| tab.path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| view.root.clone());
    if let Err(error) = view.tabs.new_file(&dir) {
        view.report(&error);
    }
    window.focus(&view.editor_focus, cx);
    cx.notify();
}

/// Cmd+S on a new file: ask where to save it. Whether the key was taken.
pub(crate) fn save_key(
    view: &mut ExplorerView,
    event: &gpui::KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) -> bool {
    let save: bool = event.keystroke.modifiers.platform && event.keystroke.key == "s";
    let untitled: Option<PathBuf> = view
        .tabs
        .active_tab()
        .filter(|tab: &&Tab| save && tab.is_untitled())
        .map(|tab: &Tab| tab.path.clone());
    let Some(path): Option<PathBuf> = untitled else {
        return false;
    };
    save_as(view, path, false, window, cx);
    cx.stop_propagation();
    true
}

/// Ask where to save the new file at `path`, then save it there. With
/// `then_close`, the close it was asked for goes on as after a plain save.
pub(crate) fn save_as(
    view: &mut ExplorerView,
    path: PathBuf,
    then_close: bool,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let dir: PathBuf = path
        .parent()
        .map_or_else(|| view.root.clone(), Path::to_path_buf);
    let name: String = path
        .file_name()
        .map_or_else(String::new, |name: &std::ffi::OsStr| {
            name.to_string_lossy().into_owned()
        });
    let pick: std::pin::Pin<Box<dyn std::future::Future<Output = Option<PathBuf>>>> =
        Box::pin(chosen(cx.prompt_for_new_path(&dir, Some(&name))));
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let chosen: Option<PathBuf> = pick.await;
            view.update_in(
                cx,
                |view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                    match chosen {
                        Some(target) => write(view, (&path, target), then_close, window, cx),
                        // Cancel: the file stays open and unsaved; a close stops.
                        None if then_close => view.cancel_close(cx),
                        None => {}
                    }
                    cx.notify();
                },
            )
            .ok();
        },
    )
    .detach();
}

/// The path picked in the save dialog; `None` on Cancel or a failed panel.
async fn chosen<F, A, B>(answer: F) -> Option<PathBuf>
where
    F: std::future::Future<Output = Result<Result<Option<PathBuf>, A>, B>>,
{
    answer.await.ok()?.ok()?
}

fn write(
    view: &mut ExplorerView,
    (old, target): (&Path, PathBuf),
    then_close: bool,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(tab): Option<&mut Tab> = view.tabs.tab_mut(old) else {
        return;
    };
    if let Err(error) = tab.document.save_as(&target) {
        view.report(&error);
        return;
    }
    let path: PathBuf = target.canonicalize().unwrap_or(target);
    view.tabs.saved_as(old, path.clone());
    crate::editor::input::schedule_parse(view, path, window, cx);
    if then_close {
        view.save_pending_close(window, cx);
    }
}

#[cfg(test)]
#[path = "untitled_tests.rs"]
mod tests;
