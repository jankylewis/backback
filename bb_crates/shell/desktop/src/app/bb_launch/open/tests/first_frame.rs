//! BB-PRF-009: a command-line launch shows its window before its language
//! walk ends; the languages, menus, servers and tab colours follow the walk.

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_lang::fixtures::{Fixture, FIXTURES};
use bb_lang::{FileMode, Lang, LangSet, Langs, Tool};
use gpui::{AnyWindowHandle, App, TestAppContext, VisualTestContext, WindowHandle};

use crate::launch::langs::{hold_walks, release_walks, walk_early, walks_of, DEBUG_DETECTING};
use crate::launch::recent::test_store;
use crate::ExplorerView;

/// Every walk is slow while this lives.
struct SlowWalks;

impl SlowWalks {
    fn hold() -> Self {
        hold_walks();
        Self
    }
}

impl Drop for SlowWalks {
    fn drop(&mut self) {
        release_walks();
    }
}

/// A scratch fixture folder and an empty choice store for one test.
struct Scratch {
    _base: tempfile::TempDir,
    _store: tempfile::TempDir,
    root: PathBuf,
}

impl Scratch {
    fn of(name: &str) -> Self {
        let store: tempfile::TempDir = tempfile::tempdir().unwrap();
        test_store(Some(store.path().to_path_buf()));
        let base: tempfile::TempDir = tempfile::tempdir().unwrap();
        let fixture: &Fixture = FIXTURES.iter().find(|f: &&Fixture| f.name == name).unwrap();
        let root: PathBuf = fixture.write(base.path()).unwrap();
        Self {
            _base: base,
            _store: store,
            root,
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        test_store(None);
    }
}

/// Launch `argument` as the command line does: the walk first, then the app.
fn launch(cx: &mut TestAppContext, argument: &Path) -> WindowHandle<ExplorerView> {
    cx.update(|cx: &mut App| {
        crate::launch::bind_keys(cx);
        cx.set_menus(crate::launch::menus(false));
    });
    let start: crate::args::Start = crate::args::start_for(argument.to_path_buf(), false);
    walk_early(&start.root, start.ticked);
    assert!(cx.update(|cx: &mut App| crate::launch::open_workspace(&start, cx)));
    cx.run_until_parked();
    cx.windows()
        .into_iter()
        .rev()
        .find_map(|w: AnyWindowHandle| w.downcast::<ExplorerView>())
        .unwrap()
}

fn detecting(window: WindowHandle<ExplorerView>, cx: &mut TestAppContext) -> bool {
    window
        .read_with(cx, |v: &ExplorerView, _: &App| v.langs.detecting)
        .unwrap()
}

fn wait_for_languages(window: WindowHandle<ExplorerView>, cx: &mut TestAppContext) {
    for _ in 0..5000 {
        cx.run_until_parked();
        if !detecting(window, cx) {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
        cx.executor().advance_clock(Duration::from_millis(4));
    }
    panic!("the languages never arrived");
}

fn menu_names(cx: &mut TestAppContext) -> Vec<String> {
    cx.update(|cx: &mut App| cx.get_menus())
        .unwrap_or_default()
        .iter()
        .map(|menu: &gpui::OwnedMenu| menu.name.to_string())
        .collect()
}

fn shown(
    window: WindowHandle<ExplorerView>,
    cx: &mut TestAppContext,
) -> (Langs, String, Vec<Tool>) {
    window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            let status: String = crate::launch::langs::status_line(v);
            (v.langs.langs, status, v.langs.started.clone())
        })
        .unwrap()
}

fn tab_mode(window: WindowHandle<ExplorerView>, cx: &mut TestAppContext) -> Option<FileMode> {
    window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            v.tabs.active_tab().map(|t: &crate::editor::Tab| t.mode)
        })
        .unwrap()
}

#[gpui::test]
fn the_window_shows_before_a_slow_walk_ends_and_takes_the_languages_after(cx: &mut TestAppContext) {
    let scratch: Scratch = Scratch::of("cs_py");
    let slow: SlowWalks = SlowWalks::hold();
    let window: WindowHandle<ExplorerView> = launch(cx, &scratch.root);
    let (before, status, started): (Langs, String, Vec<Tool>) = shown(window, cx);
    assert!(detecting(window, cx), "the window waited for the walk");
    assert_eq!(status, "Detecting languages…");
    assert!(before.active().is_empty(), "{before:?}");
    assert_eq!(
        started,
        Vec::<Tool>::new(),
        "no tool starts before the walk ends"
    );
    assert!(!menu_names(cx).contains(&String::from("Debug")));
    let mut vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    vcx.update(|w: &mut gpui::Window, _: &mut App| w.activate_window());
    drop(slow);
    wait_for_languages(window, cx);
    let held: LangSet = LangSet::of(&[Lang::CSharp, Lang::Python]);
    let (after, _, started): (Langs, String, Vec<Tool>) = shown(window, cx);
    assert_eq!(after, Langs::new(held, held));
    assert!(started.contains(&Tool::Roslyn), "{started:?}");
    let names: Vec<String> = menu_names(cx);
    assert!(names.contains(&String::from("C#")), "{names:?}");
    assert!(names.contains(&String::from("Debug")), "{names:?}");
    assert_eq!(walks_of(&scratch.root).len(), 1, "one walk of the folder");
}

#[gpui::test]
fn a_rust_file_opened_before_the_walk_ends_is_plain_then_rust(cx: &mut TestAppContext) {
    let scratch: Scratch = Scratch::of("py_rust");
    let slow: SlowWalks = SlowWalks::hold();
    let window: WindowHandle<ExplorerView> = launch(cx, &scratch.root.join("src/main.rs"));
    assert!(detecting(window, cx), "the window waited for the walk");
    assert_eq!(tab_mode(window, cx), Some(FileMode::Plain));
    drop(slow);
    wait_for_languages(window, cx);
    assert_eq!(tab_mode(window, cx), Some(FileMode::Colours(Lang::Rust)));
    let (_, status, started): (Langs, String, Vec<Tool>) = shown(window, cx);
    assert!(started.contains(&Tool::RustAnalyzer), "{started:?}");
    assert!(status.starts_with("Rust · Python"), "{status}");
}

#[gpui::test]
fn f5_before_the_walk_ends_says_the_languages_are_being_detected(cx: &mut TestAppContext) {
    let scratch: Scratch = Scratch::of("cs_only");
    let slow: SlowWalks = SlowWalks::hold();
    let window: WindowHandle<ExplorerView> = launch(cx, &scratch.root);
    let mut vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    vcx.simulate_keystrokes("f5");
    vcx.run_until_parked();
    let said: Option<String> = window
        .read_with(cx, |v: &ExplorerView, _: &App| v.status.clone())
        .unwrap();
    drop(slow);
    wait_for_languages(window, cx);
    assert_eq!(said.as_deref(), Some(DEBUG_DETECTING));
}

#[gpui::test]
fn the_start_page_path_with_ticks_knows_its_languages_at_once(cx: &mut TestAppContext) {
    let scratch: Scratch = Scratch::of("cs_rust");
    let rust: LangSet = LangSet::of(&[Lang::Rust]);
    let mut start: crate::args::Start = crate::args::start_for(scratch.root.clone(), false);
    start.ticked = Some(rust);
    assert!(cx.update(|cx: &mut App| crate::launch::open_workspace(&start, cx)));
    let window: WindowHandle<ExplorerView> = cx
        .windows()
        .into_iter()
        .find_map(|w: AnyWindowHandle| w.downcast::<ExplorerView>())
        .unwrap();
    assert!(!detecting(window, cx));
    let (langs, _, started): (Langs, String, Vec<Tool>) = shown(window, cx);
    let held: LangSet = LangSet::of(&[Lang::CSharp, Lang::Rust]);
    assert_eq!(langs.active(), rust, "{langs:?} of {held:?}");
    assert!(started.contains(&Tool::RustAnalyzer), "{started:?}");
    assert_eq!(
        walks_of(&scratch.root),
        vec![std::thread::current().id()],
        "the start page path walks as before"
    );
}
