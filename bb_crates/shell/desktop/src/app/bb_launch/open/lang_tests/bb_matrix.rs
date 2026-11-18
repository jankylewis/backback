//! The 49 combinations in real windows: tools started (rust-analyzer and
//! cargo with Rust active, BBEOD27; pyright, and pytest on the Tests panel,
//! with Python active, BBEOD28), Debug, tab colours.

use std::path::{Path, PathBuf};

use bb_lang::fixtures::{Fixture, FIXTURES};
use bb_lang::{FileMode, Lang, LangSet, Tool};
use gpui::{App, TestAppContext, WindowHandle};

use super::{close_all, open_with};
use crate::ExplorerView;

fn expected_started(active: LangSet, panel: bool) -> Vec<Tool> {
    let csharp: bool = active.has(Lang::CSharp);
    let mut at_open: Vec<Tool> = if csharp {
        vec![Tool::Roslyn]
    } else {
        Vec::new()
    };
    if active.has(Lang::Rust) {
        at_open.push(Tool::RustAnalyzer);
    }
    if active.has(Lang::Python) {
        at_open.push(Tool::Pyright);
    }
    if active.has(Lang::Rust) {
        at_open.push(Tool::Cargo);
    }
    let mut on_panel: Vec<Tool> = Vec::new();
    if csharp && panel {
        on_panel.push(Tool::TestDiscovery);
    }
    if active.has(Lang::Python) && panel {
        on_panel.push(Tool::Pytest);
    }
    at_open.into_iter().chain(on_panel).collect()
}

fn tab_modes(
    cx: &mut TestAppContext,
    window: WindowHandle<ExplorerView>,
    files: &[PathBuf],
) -> Vec<FileMode> {
    for file in files {
        window
            .update(
                cx,
                |v: &mut ExplorerView,
                 w: &mut gpui::Window,
                 cx: &mut gpui::Context<ExplorerView>| {
                    v.open_file(file, w, cx);
                },
            )
            .unwrap();
        cx.run_until_parked();
    }
    window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            v.tabs
                .items()
                .iter()
                .map(|t: &crate::editor::Tab| t.mode)
                .collect()
        })
        .unwrap()
}

fn debug_available(cx: &mut TestAppContext, window: WindowHandle<ExplorerView>) -> bool {
    window
        .update(
            cx,
            |_: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
                w.is_action_available(&crate::run::bb_debug::StartOrContinue, cx)
            },
        )
        .unwrap()
}

fn tools_and_debug_wrong(
    cx: &mut TestAppContext,
    window: WindowHandle<ExplorerView>,
    active: LangSet,
    case: &str,
) -> Vec<String> {
    let mut wrong: Vec<String> = Vec::new();
    window
        .update(
            cx,
            |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
                crate::tests_panel::open(v, w, cx);
                crate::run::bb_debug::session::control::start_or_continue(v, w, cx);
            },
        )
        .unwrap();
    cx.run_until_parked();
    let after: Vec<Tool> = window
        .read_with(cx, |v: &ExplorerView, _: &App| v.langs.started.clone())
        .unwrap();
    if after != expected_started(active, true) {
        wrong.push(format!("{case}: after Tests and Debug {after:?}"));
    }
    let said: Option<String> = window
        .read_with(cx, |v: &ExplorerView, _: &App| v.status.clone())
        .unwrap();
    let refused: bool = said.as_deref() == Some(super::super::langs::NO_DEBUG);
    if refused == active.has(Lang::CSharp) {
        wrong.push(format!("{case}: Debug said {said:?}"));
    }
    if debug_available(cx, window) != active.has(Lang::CSharp) {
        wrong.push(format!("{case}: debug key {}", !active.has(Lang::CSharp)));
    }
    wrong
}

fn tabs_wrong(
    cx: &mut TestAppContext,
    (window, root): (WindowHandle<ExplorerView>, &Path),
    fixture: &Fixture,
    active: LangSet,
    case: &str,
) -> Vec<String> {
    let mut wrong: Vec<String> = Vec::new();
    let files: Vec<PathBuf> = fixture
        .sources()
        .iter()
        .map(|(f, _): &(&str, Lang)| root.join(f))
        .collect();
    let want: Vec<FileMode> = fixture
        .sources()
        .iter()
        .map(|(_, l): &(&str, Lang)| {
            if active.has(*l) {
                FileMode::Colours(*l)
            } else {
                FileMode::Plain
            }
        })
        .collect();
    let modes: Vec<FileMode> = tab_modes(cx, window, &files);
    if modes != want {
        wrong.push(format!("{case}: tabs {modes:?}"));
    }
    wrong
}

fn check_window(
    cx: &mut TestAppContext,
    fixture: &Fixture,
    root: &Path,
    ticked: LangSet,
) -> Vec<String> {
    let active: LangSet = ticked.and(LangSet::of(fixture.langs));
    let case: String = format!("{} ticked [{}]", fixture.name, ticked.names());
    let window: WindowHandle<ExplorerView> = open_with(cx, root, ticked);
    let mut wrong: Vec<String> = Vec::new();
    let at_open: Vec<Tool> = window
        .read_with(cx, |v: &ExplorerView, _: &App| v.langs.started.clone())
        .unwrap();
    if at_open != expected_started(active, false) {
        wrong.push(format!("{case}: at open {at_open:?}"));
    }
    wrong.extend(tools_and_debug_wrong(cx, window, active, &case));
    wrong.extend(tabs_wrong(cx, (window, root), fixture, active, &case));
    close_all(cx);
    wrong
}

#[gpui::test]
fn every_tick_set_on_every_fixture_opens_a_window_with_only_its_tools_colours_and_debug(
    cx: &mut TestAppContext,
) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut passed: usize = 0;
    let mut wrong: Vec<String> = Vec::new();
    for fixture in &FIXTURES {
        let root: PathBuf = fixture.write(base.path()).unwrap();
        for ticked in LangSet::non_empty() {
            let failures: Vec<String> = check_window(cx, fixture, &root, ticked);
            passed = passed.saturating_add(usize::from(failures.is_empty()));
            wrong.extend(failures);
        }
    }
    eprintln!("bb_lang_window_matrix passed={passed} of 49");
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert_eq!(passed, 49);
}

#[gpui::test]
fn csharp_only_opens_with_roslyn_and_nothing_else_and_its_status_says_csharp(
    cx: &mut TestAppContext,
) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = FIXTURES[0].write(base.path()).unwrap();
    let window: WindowHandle<ExplorerView> = open_with(cx, &root, LangSet::of(&[Lang::CSharp]));
    let (started, langs, status): (Vec<Tool>, bb_lang::Langs, String) = window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            (
                v.langs.started.clone(),
                v.langs.langs,
                crate::launch::langs::status_line(v),
            )
        })
        .unwrap();
    assert_eq!(started, vec![Tool::Roslyn]);
    assert_eq!(
        langs.tools_at_open(),
        bb_lang::Langs::csharp_only().tools_at_open()
    );
    assert!(status.starts_with("C# · "), "{status}");
    close_all(cx);
}

#[gpui::test]
fn a_ticked_language_the_folder_lacks_starts_nothing_and_the_status_says_so(
    cx: &mut TestAppContext,
) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = FIXTURES[0].write(base.path()).unwrap();
    let window: WindowHandle<ExplorerView> = open_with(cx, &root, LangSet::of(&[Lang::Rust]));
    let (started, status): (Vec<Tool>, String) = window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            (
                v.langs.started.clone(),
                crate::launch::langs::status_line(v),
            )
        })
        .unwrap();
    assert!(started.is_empty(), "{started:?}");
    assert_eq!(status, "No Rust project here");
    close_all(cx);
}
