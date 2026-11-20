//! BBEOD41: a C# and Python folder opened with no saved choice ignores
//! Python in the whole window: a `.py` tab is plain with no PY mark and
//! reaches no server, the status bar and menus say nothing of Python,
//! Python Check refuses, the Tests panel scans no pytest root and Find in
//! Files offers no Python project.

use std::path::{Path, PathBuf};

use bb_lang::{FileMode, Tool};
use gpui::{App, TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::close_all;
use super::opt_in::{Starts, Unpicked};
use crate::intel::bb_worker::protocol::Request;
use crate::ExplorerView;

const PY: &str = "tool/main.py";

type Win = (WindowHandle<ExplorerView>, VisualTestContext);

fn window_on(cx: &mut TestAppContext, folder: &Unpicked) -> Win {
    let window: WindowHandle<ExplorerView> = folder.open(cx);
    let mut vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    vcx.update(|w: &mut gpui::Window, cx: &mut App| {
        crate::launch::bind_keys(cx);
        w.activate_window();
    });
    vcx.run_until_parked();
    (window, vcx)
}

fn read<T>((window, vcx): &mut Win, read: impl FnOnce(&ExplorerView) -> T) -> T {
    window
        .read_with(vcx, |v: &ExplorerView, _: &App| read(v))
        .unwrap()
}

fn act(
    (window, vcx): &mut Win,
    act: impl FnOnce(&mut ExplorerView, &mut gpui::Window, &mut gpui::Context<ExplorerView>),
) {
    window.update(vcx, act).unwrap();
    vcx.run_until_parked();
}

fn opened_paths(starts: &Starts) -> Vec<PathBuf> {
    starts
        .0
        .opens
        .borrow_mut()
        .iter_mut()
        .flat_map(|rx: &mut tokio::sync::mpsc::UnboundedReceiver<Request>| {
            std::iter::from_fn(|| rx.try_recv().ok()).collect::<Vec<Request>>()
        })
        .filter_map(|r: Request| {
            if let Request::Open { path, .. } = r {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

#[gpui::test]
fn an_unpicked_python_file_opens_plain_with_no_mark_and_reaches_no_server(cx: &mut TestAppContext) {
    let folder: Unpicked = Unpicked::cs_py();
    let starts: Starts = Starts::install();
    let mut win: Win = window_on(cx, &folder);
    let py: PathBuf = folder.root.join(PY);
    act(
        &mut win,
        |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            v.open_file(&py, w, cx);
        },
    );
    let mode: Option<FileMode> = read(&mut win, |v: &ExplorerView| {
        v.tabs.active_tab().map(|t: &crate::editor::Tab| t.mode)
    });
    assert_eq!(mode, Some(FileMode::Plain));
    assert!(win.1.debug_bounds("tab-mark-0-PY").is_none(), "no PY mark");
    assert!(
        !opened_paths(&starts).contains(&py),
        "no server got the .py file"
    );
    let status: String = read(&mut win, crate::launch::langs::status_line);
    assert!(!status.contains("Python"), "{status}");
    close_all(cx);
}

fn menu_items(cx: &mut TestAppContext) -> Vec<String> {
    cx.update(|cx: &mut App| cx.get_menus())
        .unwrap_or_default()
        .iter()
        .flat_map(|m: &gpui::OwnedMenu| m.items.iter())
        .filter_map(|item: &gpui::OwnedMenuItem| {
            if let gpui::OwnedMenuItem::Action { name, .. } = item {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
}

#[gpui::test]
fn an_unpicked_window_has_no_python_menu_item_and_python_check_refuses(cx: &mut TestAppContext) {
    let folder: Unpicked = Unpicked::cs_py();
    let _starts: Starts = Starts::install();
    let mut win: Win = window_on(cx, &folder);
    let items: Vec<String> = menu_items(cx);
    assert!(items.contains(&String::from("Build Solution")), "{items:?}");
    assert!(!items.contains(&String::from("Python Check")), "{items:?}");
    act(
        &mut win,
        |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::run::build::on_python_check(v, &crate::run::build::PythonCheck, w, cx);
        },
    );
    let said: Option<String> = read(&mut win, |v: &ExplorerView| v.status.clone());
    assert_eq!(said.as_deref(), Some("Build: Python is not active here"));
    close_all(cx);
}

#[gpui::test]
fn the_tests_panel_of_an_unpicked_window_scans_no_pytest_root(cx: &mut TestAppContext) {
    let folder: Unpicked = Unpicked::cs_py();
    let _starts: Starts = Starts::install();
    let mut win: Win = window_on(cx, &folder);
    act(
        &mut win,
        |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            crate::tests_panel::open(v, w, cx);
        },
    );
    let (started, projects): (Vec<Tool>, Vec<PathBuf>) = read(&mut win, |v: &ExplorerView| {
        (
            v.langs.started.clone(),
            v.tests.model.projects().keys().cloned().collect(),
        )
    });
    assert!(!started.contains(&Tool::Pytest), "{started:?}");
    let pyproject: PathBuf = folder.root.join("pyproject.toml");
    assert!(!projects.contains(&pyproject), "{projects:?}");
    close_all(cx);
}

fn write(root: &Path, file: &str, text: &str) {
    let path: PathBuf = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[gpui::test]
fn find_in_files_of_an_unpicked_window_offers_no_python_project(cx: &mut TestAppContext) {
    let folder: Unpicked = Unpicked::cs_py();
    write(
        &folder.root,
        "svc/pyproject.toml",
        "[project]\nname = \"svc\"\n",
    );
    write(&folder.root, "svc/app.py", "x = 1\n");
    write(
        &folder.root,
        "lib/pyproject.toml",
        "[project]\nname = \"lib\"\n",
    );
    let _starts: Starts = Starts::install();
    let mut win: Win = window_on(cx, &folder);
    let app: PathBuf = folder.root.join("svc/app.py");
    act(
        &mut win,
        |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
            v.open_file(&app, w, cx);
        },
    );
    win.1.dispatch_action(crate::quick_open::FindInFiles);
    win.1.run_until_parked();
    let places: Vec<String> = read(&mut win, |v: &ExplorerView| {
        v.quick_open
            .as_ref()
            .map_or_else(Vec::new, |q: &crate::quick_open::QuickOpen| {
                q.matches()
                    .iter()
                    .map(|p: &&Path| p.display().to_string())
                    .collect()
            })
    });
    assert!(
        places
            .iter()
            .any(|p: &String| p.starts_with("Project · App")),
        "{places:?}"
    );
    assert!(
        !places
            .iter()
            .any(|p: &String| p.starts_with("Project · svc") || p.starts_with("Project · lib")),
        "{places:?}"
    );
    close_all(cx);
}

#[gpui::test]
fn the_explorer_rows_of_an_unpicked_window_draw_python_project_files_plain(
    cx: &mut TestAppContext,
) {
    let folder: Unpicked = Unpicked::cs_py();
    let _starts: Starts = Starts::install();
    let mut win: Win = window_on(cx, &folder);
    let python: bool = read(&mut win, |v: &ExplorerView| {
        crate::panel::row_state(v, 0).python
    });
    assert!(!python, "pyproject.toml gets no build icon");
    close_all(cx);
}

#[test]
fn python_project_files_get_the_build_icon_only_while_python_is_active() {
    use crate::icons::{file_type, FileType};
    let cases: [(&str, FileType); 3] = [
        ("pyproject.toml", FileType::Config),
        ("setup.py", FileType::Generic),
        ("requirements.txt", FileType::Text),
    ];
    for (name, plain) in cases {
        assert_eq!(file_type(name, true), FileType::Build, "{name}");
        assert_eq!(file_type(name, false), plain, "{name}");
    }
}
