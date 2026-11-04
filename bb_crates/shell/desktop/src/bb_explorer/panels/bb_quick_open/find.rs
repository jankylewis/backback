//! Find in Files (Cmd+Shift+F): type a query, Enter searches the root on a
//! background thread (`bb-search`); hits fill the panel under the editor and
//! a click goes to the place. `/re/` is a regex; a capital letter makes the
//! search case-sensitive. A new search stops the one still running.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bb_intel::{DocPath, DocUri, Location, Span};
use bb_search::{Found, Hit, Query};
use bb_text::LspPoint;
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

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
    view.quick_open = Some(QuickOpen::find_box());
    window.focus(&view.quick_focus, cx);
    cx.notify();
}

/// Starts a search for `text`; a bad query only reports.
pub(crate) fn run(
    view: &mut ExplorerView,
    text: &str,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let query: Query = match Query::parse(text) {
        Ok(query) => query,
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
    view.status = Some(format!("Searching for {text}..."));
    let root: std::path::PathBuf = view.root.clone();
    let started: Instant = Instant::now();
    let search: gpui::Task<Found> = cx
        .background_executor()
        .spawn(async move { bb_search::search(&root, &query, CAP, &stop) });
    let label: String = text.to_owned();
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let found: Found = search.await;
            let _closed: gpui::Result<()> = view.update(
                cx,
                |v: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                    finish(v, &label, &found, started.elapsed());
                    cx.notify();
                },
            );
        },
    )
    .detach();
}

/// Fills the panel; a stopped search leaves it to the newer one.
fn finish(view: &mut ExplorerView, label: &str, found: &Found, took: Duration) {
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
    let title: String = format!("{hits}{capped} for {label} in {} files", found.files);
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
