//! A window's languages: the start page's ticks or the saved choice, then
//! what the folder holds. A command-line launch walks its folder on its own
//! thread (BB-PRF-007); the window never waits for that walk: it shows at
//! once and takes the languages when the walk ends (BB-PRF-009).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bb_lang::{default_ticks, detect, detect_until, Detection, LangSet, Langs, DETECT_BUDGET};
use gpui::{AsyncWindowContext, Context, WeakEntity, Window};
use tokio::sync::oneshot;

use super::WindowLangs;
use crate::ExplorerView;

/// The languages a window opens with: known now, or when the walk ends.
pub(crate) enum Resolved {
    Now(Langs),
    Later(Pending),
}

/// A launch's walk still running when its window opened.
pub(crate) struct Pending {
    root: PathBuf,
    ticks: Option<LangSet>,
    found: oneshot::Receiver<Detection>,
}

/// A walk started before the app ran: its folder, its ticks, its answer.
struct Early {
    root: PathBuf,
    ticks: Option<LangSet>,
    found: oneshot::Receiver<Detection>,
}

static EARLY: Mutex<Option<Early>> = Mutex::new(None);

/// The ticks a window opening `root` uses: `ticked` from the start page
/// (saved for next time), else the saved choice, else the languages the
/// folder holds (saved once the walk saw the whole folder; C# alone for a
/// folder that holds none, unsaved). With ticks known, the walk stops once
/// each ticked language is seen. A launch's walk still running is `Later`;
/// every other open walks here, as before (the start page path).
pub(crate) fn resolve(root: &Path, ticked: Option<LangSet>) -> Resolved {
    if let Some(ticks) = ticked {
        crate::launch::recent::save_langs(root, ticks);
    }
    let ticks: Option<LangSet> = ticked.or_else(|| crate::launch::recent::langs_of(root));
    let Some(mut found): Option<oneshot::Receiver<Detection>> = early_walk(root, ticks) else {
        return Resolved::Now(finish(root, ticks, walk(root, ticks)));
    };
    match found.try_recv() {
        Ok(done) => Resolved::Now(finish(root, ticks, done)),
        Err(oneshot::error::TryRecvError::Empty) => Resolved::Later(Pending {
            root: root.to_path_buf(),
            ticks,
            found,
        }),
        Err(oneshot::error::TryRecvError::Closed) => {
            Resolved::Now(finish(root, ticks, walk(root, ticks)))
        }
    }
}

/// The window's languages from a walk's answer; a complete walk of a folder
/// with no saved choice is saved. Called on the UI thread.
fn finish(root: &Path, ticks: Option<LangSet>, found: Detection) -> Langs {
    if let Some(ticks) = ticks {
        return Langs::new(ticks, found.found);
    }
    if found.complete && !found.found.is_empty() {
        crate::launch::recent::save_langs(root, found.found);
    }
    Langs::new(default_ticks(found.found), found.found)
}

/// The walk [`resolve`] needs: up to the known ticks, else the whole folder.
fn walk(root: &Path, ticks: Option<LangSet>) -> Detection {
    #[cfg(test)]
    super::record_walk(root);
    #[cfg(test)]
    wait_while_held();
    match ticks {
        Some(ticks) => detect_until(root, ticks, DETECT_BUDGET),
        None => detect(root, DETECT_BUDGET),
    }
}

/// A command-line launch: start the walk of `root` on its own thread now, so
/// it runs while the app and the window start (BB-PRF-007). The saved choice
/// is read here, on the calling thread, as [`resolve`] reads it.
pub(crate) fn walk_early(root: &Path, ticked: Option<LangSet>) {
    let ticks: Option<LangSet> = ticked.or_else(|| crate::launch::recent::langs_of(root));
    let owned: PathBuf = root.to_path_buf();
    let (send, found): (oneshot::Sender<Detection>, oneshot::Receiver<Detection>) =
        oneshot::channel();
    let spawned: std::io::Result<std::thread::JoinHandle<()>> = std::thread::Builder::new()
        .name(String::from("bb-detect"))
        .spawn(move || {
            // A window closed before the walk ended no longer listens.
            let _unheard: Result<(), Detection> = send.send(walk(&owned, ticks));
        });
    if spawned.is_err() {
        return;
    }
    if let Ok(mut slot) = EARLY.lock() {
        *slot = Some(Early {
            root: root.to_path_buf(),
            ticks,
            found,
        });
    }
}

/// The early walk's answer when it was for `root` with `ticks`.
fn early_walk(root: &Path, ticks: Option<LangSet>) -> Option<oneshot::Receiver<Detection>> {
    let early: Early = EARLY.lock().ok()?.take()?;
    (early.root == root && early.ticks == ticks).then_some(early.found)
}

/// How often a waiting window looks for its walk's answer. A wake from the
/// walk's own thread is not one gpui's test scheduler can order.
const POLL: std::time::Duration = std::time::Duration::from_millis(4);

/// When `pending`'s walk ends, the window takes its languages and starts
/// what they allow ([`crate::launch::languages_found`]).
pub(crate) fn when_found(pending: Pending, window: &mut Window, cx: &mut Context<ExplorerView>) {
    cx.spawn_in(
        window,
        async move |view: WeakEntity<ExplorerView>, cx: &mut AsyncWindowContext| {
            let Pending {
                root,
                ticks,
                mut found,
            }: Pending = pending;
            let done: Detection = loop {
                match found.try_recv() {
                    Ok(done) => break done,
                    Err(oneshot::error::TryRecvError::Empty) => {
                        cx.background_executor().timer(POLL).await;
                    }
                    Err(oneshot::error::TryRecvError::Closed) => {
                        let again: PathBuf = root.clone();
                        break cx
                            .background_executor()
                            .spawn(async move { walk(&again, ticks) })
                            .await;
                    }
                }
            };
            let _closed: gpui::Result<()> = view.update_in(
                cx,
                |view: &mut ExplorerView, window: &mut Window, cx: &mut Context<ExplorerView>| {
                    view.langs = WindowLangs::new(finish(&root, ticks, done));
                    crate::launch::languages_found(view, window, cx);
                },
            );
        },
    )
    .detach();
}

/// Tests: a slow walk. Every walk waits until [`release_walks`], 3 s at
/// most, so a window that waits for its walk is late, not hung.
#[cfg(test)]
static HELD: Mutex<bool> = Mutex::new(false);

#[cfg(test)]
pub(crate) fn hold_walks() {
    if let Ok(mut held) = HELD.lock() {
        *held = true;
    }
}

#[cfg(test)]
pub(crate) fn release_walks() {
    if let Ok(mut held) = HELD.lock() {
        *held = false;
    }
}

#[cfg(test)]
fn wait_while_held() {
    let since: std::time::Instant = std::time::Instant::now();
    while HELD
        .lock()
        .is_ok_and(|held: std::sync::MutexGuard<'_, bool>| *held)
        && since.elapsed() < std::time::Duration::from_secs(3)
    {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
