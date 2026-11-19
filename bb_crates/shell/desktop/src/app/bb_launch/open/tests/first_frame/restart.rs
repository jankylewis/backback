//! BBEOD39, the `resync_open` family: after Roslyn's thread stops, Restart
//! Server (Cmd+Shift+R) sends each C# tab to the new Roslyn once, and
//! nothing to rust-analyzer or pyright, which still hold their files open
//! (a second `didOpen` there fails as "already open").

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use bb_intel::Server;
use bb_lang::{Lang, LangSet};
use gpui::{AnyWindowHandle, App, TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::lang_tests::tools::live::bb_kit::{Bridge, Live};
use super::bb_late::{installed, workspace, Probe, PROBES};
use super::relay::{fake_starts, opens_per_start, starter, Requests, Shared, Starts};
use super::Scratch;
use crate::intel::bb_worker::protocol::Event;
use crate::intel::model::LoadStatus;
use crate::ExplorerView;

type Open = (usize, Server, PathBuf, String);

fn opens(log: &Shared) -> Vec<Open> {
    log.lock().unwrap().opens.clone()
}

/// A window with C#, Rust and Python ticked: every server starts at once.
fn all_three(cx: &mut TestAppContext, scratch: &Scratch) -> WindowHandle<ExplorerView> {
    let mut start: crate::args::Start = crate::args::start_for(scratch.root.clone(), false);
    start.ticked = Some(LangSet::of(&[Lang::CSharp, Lang::Rust, Lang::Python]));
    cx.update(|cx: &mut App| crate::launch::bind_keys(cx));
    assert!(cx.update(|cx: &mut App| crate::launch::open_workspace(&start, cx)));
    cx.run_until_parked();
    cx.windows()
        .into_iter()
        .find_map(|w: AnyWindowHandle| w.downcast::<ExplorerView>())
        .unwrap()
}

/// Roslyn's thread ends as after a crash: its link is cut and the window
/// hears `Stopped`.
fn stop_roslyn(live: &mut Live, log: &Shared) {
    let roslyn: usize = opens(log)
        .iter()
        .find(|o: &&Open| o.1 == Server::Roslyn)
        .map(|o: &Open| o.0)
        .unwrap();
    let cut: Option<crate::intel::bb_worker::Link> =
        log.lock().unwrap().links[roslyn].lock().unwrap().take();
    drop(cut);
    live.bridges.borrow()[roslyn]
        .1
        .send(Event::Stopped)
        .unwrap();
    live.until("C# stopped", &mut |_: &mut Live| {}, &|v: &ExplorerView| {
        v.intel.status == LoadStatus::Stopped
    });
}

/// No file opened twice on one server start; the C# tab on both Roslyns;
/// no sync failure.
fn once_per_server(live: &Live, log: &Shared) {
    let mut sent: Vec<(usize, PathBuf)> =
        opens(log).into_iter().map(|o: Open| (o.0, o.2)).collect();
    let all: usize = sent.len();
    sent.sort();
    sent.dedup();
    assert_eq!(
        sent.len(),
        all,
        "a file opened twice on one server: {sent:?}"
    );
    let cs: PathBuf = live.root.join(PROBES[0].file);
    let on: usize = sent
        .iter()
        .filter(|(_, p): &&(usize, PathBuf)| *p == cs)
        .count();
    assert_eq!(on, 2, "the C# tab reaches the new Roslyn: {sent:?}");
    assert_eq!(log.lock().unwrap().sync_failed, Vec::<String>::new());
}

#[gpui::test]
fn a_roslyn_restart_reopens_only_the_csharp_tabs(cx: &mut TestAppContext) {
    let scratch: Scratch = Scratch::written(&mut |base: &std::path::Path| workspace(base));
    let (log, bridges): (Shared, Rc<RefCell<Vec<Bridge>>>) = (Shared::default(), Rc::default());
    let _starts: Starts = Starts::install(starter(&log, &bridges));
    let window: WindowHandle<ExplorerView> = all_three(cx, &scratch);
    let vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    let mut live: Live = Live::new(scratch.root.clone(), window, vcx);
    live.bridges = Rc::clone(&bridges);
    for probe in &PROBES {
        live.open(probe.file);
    }
    if PROBES.iter().all(|p: &Probe| installed(p.server)) {
        let first: &Shared = &log;
        live.until(
            "each file opened",
            &mut |_: &mut Live| {},
            &|_: &ExplorerView| opens(first).len() >= PROBES.len(),
        );
        stop_roslyn(&mut live, &log);
        live.keys("cmd-shift-r");
        let settle: Instant = Instant::now();
        live.until(
            "3 s after the restart",
            &mut |_: &mut Live| {},
            &|_: &ExplorerView| settle.elapsed() > Duration::from_secs(3),
        );
        once_per_server(&live, &log);
    }
    super::close_window(window, cx);
    super::relay::wait_ended(&log);
}

/// Tab `file` lands (load done, its open event not yet flushed), then
/// `then` runs in the same update; the didOpens each fake server got.
fn lands_then(
    cx: &mut TestAppContext,
    (window, file): (WindowHandle<ExplorerView>, &std::path::Path),
    then: &mut dyn FnMut(&mut ExplorerView, &mut gpui::Window, &mut gpui::Context<ExplorerView>),
    sent: &Rc<RefCell<Vec<Requests>>>,
) -> Vec<usize> {
    window
        .update(
            cx,
            |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
                let _placeholder: Option<PathBuf> = v.tabs.begin_open(file).unwrap();
                let loaded: Result<crate::editor::Tab, crate::editor::TabError> =
                    crate::editor::Tab::load(file.to_path_buf(), bb_lang::FileMode::Plain);
                v.finish_open(file, loaded, w, cx);
                then(v, w, cx);
            },
        )
        .unwrap();
    cx.run_until_parked();
    let counts: Vec<usize> = opens_per_start(sent);
    super::close_window(window, cx);
    counts
}

/// A tab that lands in the same update as the walk's answer, before any
/// frame flushes its open event, reaches the new server once.
#[gpui::test]
fn a_tab_landing_in_the_answers_update_is_opened_once(cx: &mut TestAppContext) {
    let scratch: Scratch = Scratch::of("py_only");
    let (_starts, sent): (Starts, Rc<RefCell<Vec<Requests>>>) = fake_starts();
    let slow: super::SlowWalks = super::SlowWalks::hold();
    let window: WindowHandle<ExplorerView> = super::launch(cx, &scratch.root);
    let python: LangSet = LangSet::of(&[Lang::Python]);
    let file: PathBuf = scratch.root.join("tool/main.py");
    let answer: &mut dyn FnMut(
        &mut ExplorerView,
        &mut gpui::Window,
        &mut gpui::Context<ExplorerView>,
    ) = &mut |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
        v.langs = crate::launch::WindowLangs::new(bb_lang::Langs::new(python, python));
        crate::launch::languages_found(v, w, cx);
    };
    assert_eq!(lands_then(cx, (window, &file), answer, &sent), vec![1]);
    drop(slow);
}

/// A C# tab that lands in the same update as Restart Server reaches the
/// new Roslyn once.
#[gpui::test]
fn a_tab_landing_in_the_restarts_update_is_opened_once(cx: &mut TestAppContext) {
    let scratch: Scratch = Scratch::of("cs_only");
    let (_starts, sent): (Starts, Rc<RefCell<Vec<Requests>>>) = fake_starts();
    let window: WindowHandle<ExplorerView> = all_three(cx, &scratch);
    let file: PathBuf = scratch.root.join("App/Program.cs");
    let restart: &mut dyn FnMut(
        &mut ExplorerView,
        &mut gpui::Window,
        &mut gpui::Context<ExplorerView>,
    ) = &mut |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
        crate::intel::glue::on_event(v, (Server::Roslyn, Event::Stopped), w, cx);
        crate::intel::glue::restart(v, w, cx);
    };
    assert_eq!(lands_then(cx, (window, &file), restart, &sent), vec![0, 1]);
}
