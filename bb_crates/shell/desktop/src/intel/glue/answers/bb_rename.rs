//! Apply a rename plan to every file it names, all or nothing. Each file
//! changes in one undo step. Files that were not open open as tabs and are
//! saved; open tabs keep the change unsaved. Read-only files, stale plans
//! and file moves refuse the whole plan before anything changes.

use std::path::PathBuf;

use bb_intel::{DocUri, FileChange, TextChange};
use bb_text::{CharEdit, CharIdx, Document};

use crate::editor::Tab;
use crate::intel::model::popups::RenamePreview;
use crate::ExplorerView;

/// Why a rename plan was refused; nothing changed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum RenameError {
    #[error("no rename plan to apply")]
    NoPlan,
    #[error("no-write mode: rename shows its preview only")]
    NoWrite,
    #[error("rename would change a read-only file: {}", .0.display())]
    ReadOnly(PathBuf),
    #[error("rename would create, move or delete files; refused")]
    FileOperation,
    #[error("rename names a document with no file: {0}")]
    Virtual(String),
    #[error("{} changed since the preview; rename again", .0.display())]
    Stale(PathBuf),
    #[error("a rename edit does not fit {}", .0.display())]
    BadRange(PathBuf),
    #[error("cannot open {}: {reason}", path.display())]
    Open { path: PathBuf, reason: String },
    #[error("save failed, every file is back as it was: {0}")]
    Save(String),
}

/// One file of the plan, checked and ready.
struct Staged {
    path: PathBuf,
    /// The tab, when the file was not open before.
    loaded: Option<Tab>,
    edits: Vec<CharEdit>,
}

/// Apply the previewed plan. Returns (files, edits) changed.
///
/// # Errors
/// See [`RenameError`]; on any error no buffer and no file has changed.
pub(crate) fn apply_rename(view: &mut ExplorerView) -> Result<(usize, usize), RenameError> {
    let preview: RenamePreview = view
        .intel
        .rename_preview
        .clone()
        .ok_or(RenameError::NoPlan)?;
    if view.tabs.no_write() {
        return Err(RenameError::NoWrite);
    }
    let mut staged: Vec<Staged> = Vec::new();
    for (path, changes) in files(&preview.changes)? {
        staged.push(stage(view, &preview, path, &changes)?);
    }
    let counts: (usize, usize) = preview.counts();
    edit_all(view, &mut staged)?;
    save_loaded(view, &mut staged)?;
    for item in staged {
        if let Some(tab) = item.loaded {
            view.tabs.adopt(tab);
        }
    }
    view.intel.rename_preview = None;
    Ok(counts)
}

/// The plan as (file, edits); any file operation or virtual document
/// refuses it.
fn files(plan: &[FileChange]) -> Result<Vec<(PathBuf, Vec<TextChange>)>, RenameError> {
    plan.iter()
        .map(|change: &FileChange| match change {
            FileChange::Edit {
                uri: DocUri::File(path),
                changes,
            } => Ok((path.as_path().to_path_buf(), changes.clone())),
            FileChange::Edit {
                uri: DocUri::Virtual(name),
                ..
            } => Err(RenameError::Virtual(name.clone())),
            FileChange::Create(_) | FileChange::Rename { .. } | FileChange::Delete(_) => {
                Err(RenameError::FileOperation)
            }
        })
        .collect()
}

fn stage(
    view: &mut ExplorerView,
    preview: &RenamePreview,
    path: PathBuf,
    changes: &[TextChange],
) -> Result<Staged, RenameError> {
    let outside: bool = !path.starts_with(&view.root);
    if let Some(tab) = view.tabs.tab_mut(&path) {
        if tab.read_only || outside {
            return Err(RenameError::ReadOnly(path));
        }
        if preview.revisions.get(&path) != Some(&tab.document.revision()) {
            return Err(RenameError::Stale(path));
        }
        let edits: Vec<CharEdit> = char_edits(&tab.document, changes, &path)?;
        return Ok(Staged {
            path,
            loaded: None,
            edits,
        });
    }
    if preview.revisions.contains_key(&path) {
        return Err(RenameError::Stale(path));
    }
    let tab: Tab =
        Tab::load(path.clone()).map_err(|e: crate::editor::TabError| RenameError::Open {
            path: path.clone(),
            reason: e.to_string(),
        })?;
    if tab.read_only || outside {
        return Err(RenameError::ReadOnly(path));
    }
    let edits: Vec<CharEdit> = char_edits(&tab.document, changes, &path)?;
    Ok(Staged {
        path,
        loaded: Some(tab),
        edits,
    })
}

/// LSP edits as char edits; a position that does not map back exactly (a
/// column past its line) means the plan does not fit this text.
fn char_edits(
    document: &Document,
    changes: &[TextChange],
    path: &std::path::Path,
) -> Result<Vec<CharEdit>, RenameError> {
    changes
        .iter()
        .map(|change: &TextChange| {
            Ok(CharEdit {
                from: exact(document, change.span.start, path)?,
                to: exact(document, change.span.end, path)?,
                text: change.new_text.clone(),
            })
        })
        .collect()
}

/// The char at `point`, only if it maps back to the same point (a column
/// past its line end does not).
fn exact(
    document: &Document,
    point: bb_text::LspPoint,
    path: &std::path::Path,
) -> Result<CharIdx, RenameError> {
    let at: CharIdx = document
        .char_of_lsp(point)
        .map_err(|_: bb_text::DocumentError| RenameError::BadRange(path.to_path_buf()))?;
    match document.lsp_point(at) {
        Ok(back) if back == point => Ok(at),
        Ok(_) | Err(_) => Err(RenameError::BadRange(path.to_path_buf())),
    }
}

fn document_of<'v>(view: &'v mut ExplorerView, item: &'v mut Staged) -> Option<&'v mut Document> {
    match item.loaded.as_mut() {
        Some(tab) => Some(&mut tab.document),
        None => view
            .tabs
            .tab_mut(&item.path)
            .map(|tab: &mut Tab| &mut tab.document),
    }
}

/// Change every buffer; if one refuses, undo the ones already changed.
fn edit_all(view: &mut ExplorerView, staged: &mut [Staged]) -> Result<(), RenameError> {
    for index in 0..staged.len() {
        let Some(item): Option<&mut Staged> = staged.get_mut(index) else {
            continue;
        };
        let edits: Vec<CharEdit> = item.edits.clone();
        let path: PathBuf = item.path.clone();
        let done: bool = document_of(view, item)
            .is_some_and(|doc: &mut Document| doc.replace_many(&edits).is_ok());
        if !done {
            undo_first(view, staged, index, false);
            return Err(RenameError::BadRange(path));
        }
    }
    Ok(())
}

/// Save the files that were not open; if one fails, put every file back.
fn save_loaded(view: &mut ExplorerView, staged: &mut [Staged]) -> Result<(), RenameError> {
    for index in 0..staged.len() {
        let saved: Result<(), bb_text::DocumentError> = staged
            .get_mut(index)
            .and_then(|item: &mut Staged| item.loaded.as_mut())
            .map_or(Ok(()), |tab: &mut Tab| tab.document.save());
        if let Err(error) = saved {
            undo_first(view, staged, staged.len(), true);
            return Err(RenameError::Save(error.to_string()));
        }
    }
    Ok(())
}

/// Undo the first `count` staged files; with `resave`, write the restored
/// text of each loaded file back to disk.
fn undo_first(view: &mut ExplorerView, staged: &mut [Staged], count: usize, resave: bool) {
    for item in staged.iter_mut().take(count) {
        let loaded: bool = item.loaded.is_some();
        if let Some(doc) = document_of(view, item) {
            // The edit was the latest undo step, so undo cannot miss it.
            let _undone: Result<(), bb_text::DocumentError> = doc.undo();
            if resave && loaded {
                // Best effort: the error that started the rollback is reported.
                let _restored: Result<(), bb_text::DocumentError> = doc.save();
            }
        }
    }
}
