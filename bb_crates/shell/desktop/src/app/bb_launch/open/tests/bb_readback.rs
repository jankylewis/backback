//! BBEOD33: the BBEOD32 readback suspects in real windows, by mouse and keys
//! (rows 2a, 3, 3a, 4: the file argument's root, the tab marks, the C#-only
//! menus).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_lang::fixtures::{Fixture, FIXTURES};
use bb_lang::Tool;
use gpui::{AnyWindowHandle, App, TestAppContext, VisualTestContext, WindowHandle};

use crate::ExplorerView;

fn fixture(name: &str) -> &'static Fixture {
    FIXTURES.iter().find(|f: &&Fixture| f.name == name).unwrap()
}

/// Launch with `argument` as the command line does, the menu bar as at launch.
fn launch_with(cx: &mut TestAppContext, argument: &Path) -> WindowHandle<ExplorerView> {
    cx.update(|cx: &mut App| {
        crate::launch::bind_keys(cx);
        cx.set_menus(crate::launch::menus(true));
    });
    let start: crate::args::Start = crate::args::start_for(argument.to_path_buf(), false);
    assert!(cx.update(|cx: &mut App| crate::launch::open_workspace(&start, cx)));
    cx.run_until_parked();
    cx.windows()
        .into_iter()
        .rev()
        .find_map(|w: AnyWindowHandle| w.downcast::<ExplorerView>())
        .unwrap()
}

fn menu_names(cx: &mut TestAppContext) -> Vec<String> {
    cx.update(|cx: &mut App| cx.get_menus())
        .unwrap_or_default()
        .iter()
        .map(|menu: &gpui::OwnedMenu| menu.name.to_string())
        .collect()
}

/// Click the Explorer row of `file` (its folders opened first).
fn click_row(window: WindowHandle<ExplorerView>, file: &Path, cx: &mut TestAppContext) {
    let row: usize = window
        .update(
            cx,
            |v: &mut ExplorerView, _: &mut gpui::Window, _: &mut gpui::Context<ExplorerView>| {
                v.tree.reveal(file).unwrap().unwrap()
            },
        )
        .unwrap();
    cx.run_until_parked();
    let mut vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    let selector: &'static str = format!("explorer-row-{row}").leak();
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_click(at, gpui::Modifiers::default());
    vcx.run_until_parked();
}

fn marks(window: WindowHandle<ExplorerView>, cx: &mut TestAppContext) -> Vec<String> {
    let mut vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    vcx.run_until_parked();
    let names: Vec<String> = window
        .read_with(&vcx, |v: &ExplorerView, _: &App| {
            v.tabs
                .items()
                .iter()
                .map(|t: &crate::editor::Tab| {
                    t.path
                        .file_name()
                        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap();
    names
        .into_iter()
        .enumerate()
        .map(|(index, name): (usize, String)| {
            let shown: Vec<&str> = ["C#", "RS", "PY"]
                .into_iter()
                .filter(|mark: &&str| {
                    let selector: &'static str = format!("tab-mark-{index}-{mark}").leak();
                    vcx.debug_bounds(selector).is_some()
                })
                .collect();
            format!("{name} [{}]", shown.join("+"))
        })
        .collect()
}

#[gpui::test]
fn a_rust_file_argument_opens_its_project_and_starts_its_servers(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = fixture("py_rust").write(base.path()).unwrap();
    let window: WindowHandle<ExplorerView> = launch_with(cx, &root.join("src/main.rs"));
    let (opened, started, tab): (PathBuf, Vec<Tool>, Option<PathBuf>) = window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            (
                v.root.clone(),
                v.langs.started.clone(),
                v.tabs
                    .active_tab()
                    .map(|t: &crate::editor::Tab| t.path.clone()),
            )
        })
        .unwrap();
    assert_eq!(opened, root, "the Explorer roots at the project");
    assert_eq!(
        started,
        vec![Tool::RustAnalyzer, Tool::Pyright, Tool::Cargo]
    );
    assert_eq!(tab, Some(root.join("src/main.rs")));
}

#[gpui::test]
fn each_tab_shows_its_own_files_language_mark(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = fixture("all3").write(base.path()).unwrap();
    let window: WindowHandle<ExplorerView> = launch_with(cx, &root.join("src/main.rs"));
    click_row(window, &root.join("tool/main.py"), cx);
    click_row(window, &root.join("notes/README.md"), cx);
    let mut vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    for keys in ["cmd-p", "P r o g r a m . c s", "enter"] {
        vcx.simulate_keystrokes(keys);
        vcx.run_until_parked();
    }
    assert_eq!(
        marks(window, cx),
        vec![
            "main.rs [RS]",
            "main.py [PY]",
            "README.md []",
            "Program.cs [C#]"
        ]
    );
}

#[gpui::test]
fn the_csharp_and_debug_menus_show_only_while_csharp_is_active(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let py_rust: PathBuf = fixture("py_rust").write(base.path()).unwrap();
    let rust: WindowHandle<ExplorerView> = launch_with(cx, &py_rust);
    let names: Vec<String> = menu_names(cx);
    assert!(!names.contains(&String::from("C#")), "{names:?}");
    assert!(!names.contains(&String::from("Debug")), "{names:?}");
    let cs: PathBuf = fixture("cs_only").write(base.path()).unwrap();
    let csharp: WindowHandle<ExplorerView> = launch_with(cx, &cs);
    let names: Vec<String> = menu_names(cx);
    assert!(names.contains(&String::from("C#")), "{names:?}");
    assert!(names.contains(&String::from("Debug")), "{names:?}");
    let mut vcx: VisualTestContext = VisualTestContext::from_window(csharp.into(), cx);
    vcx.deactivate_window();
    vcx.run_until_parked();
    let rust_window: AnyWindowHandle = rust.into();
    let mut rcx: VisualTestContext = VisualTestContext::from_window(rust_window, cx);
    rcx.update(|w: &mut gpui::Window, _: &mut App| w.activate_window());
    rcx.run_until_parked();
    let names: Vec<String> = menu_names(cx);
    assert!(
        !names.contains(&String::from("C#")),
        "back in the Rust window: {names:?}"
    );
}

fn wait_for_members(window: WindowHandle<ExplorerView>, vcx: &mut VisualTestContext) {
    let _armed: crate::launch::open::lang_tests::watchdog::Watchdog =
        crate::launch::open::lang_tests::watchdog::watchdog(
            "cargo metadata",
            Duration::from_secs(120),
        );
    let started: Instant = Instant::now();
    loop {
        vcx.run_until_parked();
        let loading: bool = window
            .read_with(vcx, |v: &ExplorerView, _: &App| {
                matches!(v.rust.members, crate::rows::cargo::CargoMembers::Loading)
            })
            .unwrap();
        if !loading {
            return;
        }
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "cargo metadata never answered"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[gpui::test]
fn a_broken_cargo_toml_shows_one_short_line_and_the_full_text_on_hover(cx: &mut TestAppContext) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = fixture("rust_only").write(base.path()).unwrap();
    let window: WindowHandle<ExplorerView> = launch_with(cx, &root);
    let mut vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    wait_for_members(window, &mut vcx);
    let section: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("cargo-members").unwrap();
    assert!(
        section.size.height <= gpui::px(64.0),
        "the heading and one line, not the whole error: {section:?}"
    );
    let line: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("cargo-error").unwrap();
    assert!(vcx.debug_bounds("path-tooltip").is_none());
    vcx.simulate_mouse_move(line.center(), None, gpui::Modifiers::default());
    vcx.executor().advance_clock(Duration::from_secs(2));
    vcx.run_until_parked();
    assert!(
        vcx.debug_bounds("path-tooltip").is_some(),
        "the hover shows the full text"
    );
    let failed: crate::rows::cargo::CargoMembers = window
        .read_with(&vcx, |v: &ExplorerView, _: &App| v.rust.members.clone())
        .unwrap();
    let (summary, full): (String, String) = match failed {
        crate::rows::cargo::CargoMembers::Failed { summary, full } => (summary, full),
        other @ (crate::rows::cargo::CargoMembers::Off
        | crate::rows::cargo::CargoMembers::Loading
        | crate::rows::cargo::CargoMembers::Ready(_)) => panic!("{other:?}"),
    };
    assert_eq!(
        summary,
        "cargo metadata: Cargo.toml:1: key with no value, expected `=`"
    );
    assert!(full.contains("1 | x"), "{full}");
}
