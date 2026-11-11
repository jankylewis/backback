//! The filter's search (Enter or a paste in it): a test of a project never
//! opened must show too. Listed projects with a matching test open at once;
//! the others are listed, one at a time, when their own `.cs` files hold the
//! filter's longest word. Nothing is listed while the filter is only typed.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use bb_search::{Filter, Query, SearchError};
use bb_testkit::Fqn;
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};

use crate::tests_panel::model::ProjectTests;
use crate::tests_panel::Scan;
use crate::ExplorerView;

/// The shortest word searched for: a shorter one is in every project.
const MIN_WORD: usize = 3;

/// Where the filter's search is.
#[derive(Debug, Default)]
pub(crate) struct Search {
    /// The filter text searched for last.
    text: String,
    /// Counts the searches: an older one's answer is dropped.
    round: u64,
    /// The sources are being read.
    finding: bool,
    /// Projects still to list, in order.
    queue: VecDeque<PathBuf>,
}

/// `filter` as the rows compare it: trimmed, ASCII lower case.
pub(crate) fn lowered(filter: &str) -> String {
    filter.trim().to_ascii_lowercase()
}

/// Whether `fqn` shows under the [`lowered`] filter; an empty one shows all.
pub(crate) fn holds(fqn: &Fqn, lowered: &str) -> bool {
    lowered.is_empty() || fqn.as_str().to_ascii_lowercase().contains(lowered)
}

/// The word a filter is searched by in the sources: its longest run of
/// letters, digits and `_` (a name out of `Ns.Class.Name(1)`).
pub(crate) fn word(filter: &str) -> Option<&str> {
    filter
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .max_by_key(|part: &&str| part.chars().count())
        .filter(|part: &&str| part.chars().count() >= MIN_WORD)
}

/// The projects among `candidates` with a `.cs` file of their own that holds
/// `word`, in any letter case. `all` is every project: a project's own files
/// leave out the folders of the projects inside its folder.
pub(crate) fn holding(candidates: &[PathBuf], all: &[PathBuf], word: &str) -> Vec<PathBuf> {
    let Ok(query): Result<Query, SearchError> = Query::parse(&word.to_lowercase()) else {
        return Vec::new();
    };
    let stop: AtomicBool = AtomicBool::new(false);
    candidates
        .iter()
        .filter(|project: &&PathBuf| {
            project.parent().is_some_and(|dir: &Path| {
                let own: Filter = own_sources(dir, all);
                !bb_search::search_in(dir, &own, &query, 1, &stop)
                    .hits
                    .is_empty()
            })
        })
        .cloned()
        .collect()
}

/// `*.cs` under `dir`, without the folders of other projects inside it.
fn own_sources(dir: &Path, all: &[PathBuf]) -> Filter {
    let mut globs: String = String::from("*.cs");
    for inside in all
        .iter()
        .filter_map(|project: &PathBuf| project.parent()?.strip_prefix(dir).ok())
        .filter(|inside: &&Path| !inside.as_os_str().is_empty())
    {
        globs.push_str(",!");
        globs.push_str(&inside.to_string_lossy());
    }
    Filter::parse(&globs)
        .or_else(|_: SearchError| Filter::parse("*.cs"))
        .unwrap_or_default()
}

/// Enter or a paste in the filter. Before the projects are known nothing
/// happens: the row's hint stays, so Enter can be pressed again.
pub(crate) fn search(view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>) {
    if view.tests.scan != Scan::Done {
        return;
    }
    let text: String = view.tests.filter.trim().to_owned();
    view.tests.search.queue.clear();
    view.tests.search.text.clone_from(&text);
    view.tests.search.round = view.tests.search.round.wrapping_add(1);
    view.tests.search.finding = false;
    view.tests.message = None;
    cx.notify();
    let Some(needle): Option<String> = word(&text).map(str::to_owned) else {
        return;
    };
    let all: Vec<PathBuf> = view.tests.model.projects().keys().cloned().collect();
    let unlisted: Vec<PathBuf> = open_listed(view, &all, &lowered(&text));
    if unlisted.is_empty() {
        return;
    }
    view.tests.search.finding = true;
    let round: u64 = view.tests.search.round;
    let job: gpui::Task<Vec<PathBuf>> = cx
        .background_executor()
        .spawn(async move { holding(&unlisted, &all, &needle) });
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let found: Vec<PathBuf> = job.await;
            view.update_in(
                cx,
                |view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                    found_projects(view, (round, found), window, cx);
                },
            )
            .ok();
        },
    )
    .detach();
}

/// Open each listed project that has a test under `lower`; the projects not
/// listed yet.
fn open_listed(view: &mut ExplorerView, all: &[PathBuf], lower: &str) -> Vec<PathBuf> {
    let mut unlisted: Vec<PathBuf> = Vec::new();
    for path in all {
        let Some(project): Option<&mut ProjectTests> = view.tests.model.project_mut(path) else {
            continue;
        };
        if project.specs.is_empty() {
            unlisted.push(path.clone());
        } else if project.specs.keys().any(|fqn: &Fqn| holds(fqn, lower)) {
            project.expanded = true;
        }
    }
    unlisted
}

/// The sources were read for search `round`: list what was `found`.
fn found_projects(
    view: &mut ExplorerView,
    (round, found): (u64, Vec<PathBuf>),
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if round != view.tests.search.round {
        return;
    }
    view.tests.search.finding = false;
    let lower: String = lowered(&view.tests.search.text);
    let shown: bool = view
        .tests
        .model
        .projects()
        .values()
        .any(|project: &ProjectTests| project.specs.keys().any(|fqn: &Fqn| holds(fqn, &lower)));
    if found.is_empty() && !shown {
        view.tests.message = Some(format!(
            "No test source holds \"{}\"",
            word(&view.tests.search.text).unwrap_or_default()
        ));
    }
    view.tests.search.queue = found.into();
    list_next(view, window, cx);
    cx.notify();
}

/// List the search's next project. One at a time: nothing starts while any
/// project is being listed, and every list that ends calls this again. A
/// filter changed since the search drops the rest.
pub(crate) fn list_next(
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    if view.tests.filter.trim() != view.tests.search.text {
        view.tests.search.queue.clear();
    }
    let listing: bool = view
        .tests
        .model
        .projects()
        .values()
        .any(|project: &ProjectTests| project.loading);
    if listing {
        return;
    }
    while let Some(path) = view.tests.search.queue.pop_front() {
        let Some(project): Option<&mut ProjectTests> = view.tests.model.project_mut(&path) else {
            continue;
        };
        project.expanded = true;
        if project.specs.is_empty() {
            super::super::start_list(view, path, window, cx);
            return;
        }
    }
}

/// What the filter row says of the search: that it runs, or how to start it
/// while some project has not been listed.
pub(crate) fn hint(view: &ExplorerView) -> Option<&'static str> {
    if view.tests.search.finding {
        return Some("Searching the sources…");
    }
    let text: &str = view.tests.filter.trim();
    let unlisted: bool = view
        .tests
        .model
        .projects()
        .values()
        .any(|project: &ProjectTests| project.specs.is_empty());
    (view.tests.search.text != text && unlisted && word(text).is_some())
        .then_some("Enter: search every project")
}
