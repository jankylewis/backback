//! Find in Files (Cmd+Shift+F): type a query, Enter searches the root on a
//! background thread (`bb-search`); hits fill the panel under the editor and
//! a click goes to the place. `/re/` is a regex; a capital letter makes the
//! search case-sensitive. A new search stops the one still running.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bb_intel::{DocPath, DocUri, Location, Span};
use bb_search::{Filter, Found, Hit, Query};
use bb_text::LspPoint;
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

use super::bb_model::FindAsk;
use super::QuickOpen;
use crate::intel::bb_worker::protocol::Place;
use crate::intel::model::popups::{Pick, PlacesPanel};
use crate::ExplorerView;

/// Hits listed at most; the title says when the cap stopped the search.
pub(crate) const CAP: usize = 2_000;

#[derive(Clone, Debug, PartialEq, Default, gpui::Action)]
#[action(namespace = backback)]
pub(crate) struct FindInFiles;

pub(crate) fn on_find_in_files(
    view: &mut ExplorerView,
    _: &FindInFiles,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    open(view, None, window, cx);
}

/// The Explorer's Find in Folder...: the box with `folder` as its place.
pub(crate) fn find_in(
    view: &mut ExplorerView,
    folder: &Path,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    open(view, Some(folder), window, cx);
}

/// Opens the box at once; the C# projects join its places after a walk
/// off the UI thread (as Go to File's, BB-PRF-002).
fn open(
    view: &mut ExplorerView,
    first: Option<&Path>,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let root: PathBuf = view.root.clone();
    let active: Option<PathBuf> = view
        .tabs
        .active_tab()
        .map(|tab: &crate::editor::Tab| tab.path.clone());
    view.quick_open = Some(QuickOpen::find_box(&root, active.as_deref(), first));
    window.focus(&view.quick_focus, cx);
    cx.notify();
    let walk_root: PathBuf = root.clone();
    let walk: gpui::Task<Vec<PathBuf>> = cx.background_executor().spawn(async move {
        QuickOpen::list_files(&walk_root)
            .into_iter()
            .filter(|file: &PathBuf| {
                file.extension()
                    .is_some_and(|e: &std::ffi::OsStr| e == "csproj")
            })
            .map(|file: PathBuf| walk_root.join(file))
            .collect()
    });
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let projects: Vec<PathBuf> = walk.await;
            let _closed: gpui::Result<()> = view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                    if let Some(quick) = v
                        .quick_open
                        .as_mut()
                        .filter(|q: &&mut QuickOpen| q.fields().is_some())
                    {
                        quick.add_projects(&root, &projects);
                        cx.notify();
                    }
                },
            );
        },
    )
    .detach();
}

/// Starts the search `ask` asks for; a bad query or glob only reports.
pub(crate) fn run(
    view: &mut ExplorerView,
    ask: &FindAsk,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let parsed: Result<(Query, Filter), bb_search::SearchError> =
        Query::with(&ask.text, ask.options)
            .and_then(|q: Query| Ok((q, Filter::parse(&ask.files)?)));
    let (query, filter): (Query, Filter) = match parsed {
        Ok(parsed) => parsed,
        Err(error) => {
            view.report(&format!("Find in Files: {error}"));
            return;
        }
    };
    if let Some(old) = view.intel.find_stop.take() {
        old.store(true, Ordering::Relaxed);
    }
    let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    view.intel.find_stop = Some(Arc::clone(&stop));
    view.status = Some(format!("Searching for {}...", ask.text));
    let dir: PathBuf = ask.dir.clone();
    let started: Instant = Instant::now();
    let search: gpui::Task<Found> = cx
        .background_executor()
        .spawn(async move { bb_search::search_in(&dir, &filter, &query, CAP, &stop) });
    let label: (String, bool) = label(ask, &view.root);
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let found: Found = search.await;
            let _closed: gpui::Result<()> = view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                    finish(v, (&label.0, label.1), &found, started.elapsed());
                    cx.notify();
                },
            );
        },
    )
    .detach();
}

/// The results title's "for ...": the text, and when the search was
/// narrowed, the place and the files; whether it was narrowed.
fn label(ask: &FindAsk, root: &Path) -> (String, bool) {
    let files: String = if ask.files.trim().is_empty() {
        String::new()
    } else {
        format!(" ({})", ask.files.trim())
    };
    let place: String = if ask.dir == root {
        String::new()
    } else {
        format!(" in {}", ask.place)
    };
    let narrowed: bool = !(files.is_empty() && place.is_empty());
    (format!("{}{place}{files}", ask.text), narrowed)
}

/// Fills the panel; a stopped search leaves it to the newer one.
fn finish(view: &mut ExplorerView, (label, narrowed): (&str, bool), found: &Found, took: Duration) {
    if found.stopped {
        return;
    }
    view.intel.find_stop = None;
    let hits: String = match found.hits.len() {
        1 => "1 result".to_owned(),
        n => format!("{n} results"),
    };
    let capped: &str = if found.capped {
        " (stopped at 2000)"
    } else {
        ""
    };
    let files: String = match found.files {
        1 => "1 file".to_owned(),
        n => format!("{n} files"),
    };
    let title: String = if narrowed {
        format!("{hits}{capped} for {label}, {files}")
    } else {
        format!("{hits}{capped} for {label} in {files}")
    };
    view.status = Some(format!("{title}, {} ms", took.as_millis()));
    view.intel.references = Some(PlacesPanel {
        title,
        places: found.hits.iter().filter_map(place).collect(),
        pick: Pick::default(),
    });
}

fn place(hit: &Hit) -> Option<Place> {
    let at: LspPoint = LspPoint {
        line: hit.line,
        character: hit.column,
    };
    Some(Place {
        location: Location {
            uri: DocUri::File(DocPath::new(hit.path.clone()).ok()?),
            span: Span { start: at, end: at },
        },
        preview: hit.preview.clone(),
    })
}

/// Go to line `line` (0-based) of the active file.
pub(super) fn go_to_line(
    view: &mut ExplorerView,
    line: usize,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(path): Option<std::path::PathBuf> = view
        .tabs
        .active_tab()
        .map(|tab: &crate::editor::Tab| tab.path.clone())
    else {
        return;
    };
    let at: bb_text::LspPoint = bb_text::LspPoint {
        line: u32::try_from(line).unwrap_or(u32::MAX),
        character: 0,
    };
    crate::intel::glue::goto_point(view, &path, at, window, cx);
}
