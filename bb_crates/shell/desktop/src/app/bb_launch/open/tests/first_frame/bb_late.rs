//! BBEOD39 (a): a C#, a Rust and a Python file opened while the launch walk
//! runs, the Python one edited, reach the REAL servers the walk's answer
//! starts: one `didOpen` each, to its own server, with the tab's text;
//! then hover, F12 and diagnostics answer for each file, the Python ones
//! on the typed line. A server that is not installed takes its branch (a
//! failure when `BACKBACK_REQUIRE_TOOLS`). The user picked all three
//! languages before (BBEOD41: Python is never on by default).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use bb_intel::Server;
use gpui::{TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::lang_tests::tools::live::bb_kit::{active, tools_required, Bridge, Live};
use super::relay::{starter, Shared, Starts};
use super::{detecting, launch, wait_for_languages, Scratch, SlowWalks};
use crate::ExplorerView;

const PY: &str = "tool/main.py";
const PY_TEXT: &str = "def add(a: int, b: int) -> int:\n    return a + b\n\n\ntotal = add(1, 2)\n";
const RS_TEXT: &str =
    "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\nfn main() {\n    let total = add(1, 2);\n    println!(\"{total}\");\n}\n";
const CARGO: &str = "[package]\nname = \"late\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const PYPROJECT: &str = "[project]\nname = \"late\"\nversion = \"0.1.0\"\n";

/// One file's questions: F12 after `f12_after` lands on `def`; a hover at
/// (line, column) says `hover_has`.
pub(super) struct Probe {
    pub(super) file: &'static str,
    pub(super) server: Server,
    f12_after: &'static str,
    def: (&'static str, usize),
    hover_at: (u32, u32),
    hover_has: &'static str,
}

pub(super) const PROBES: [Probe; 3] = [
    Probe {
        file: "_NUnitApex/GreeterSpec.cs",
        server: Server::Roslyn,
        f12_after: "new Greet",
        def: ("Core/Greeter.cs", 2),
        hover_at: (12, 31),
        hover_has: "Greeter",
    },
    Probe {
        file: "src/main.rs",
        server: Server::RustAnalyzer,
        f12_after: "= ad",
        def: ("src/main.rs", 0),
        hover_at: (5, 17),
        hover_has: "fn add",
    },
    Probe {
        file: PY,
        server: Server::Pyright,
        f12_after: "= ad",
        def: (PY, 0),
        hover_at: (4, 9),
        hover_has: "def add",
    },
];

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path: PathBuf = entry.unwrap().path();
        let name: std::ffi::OsString = path.file_name().unwrap().to_os_string();
        if path.is_dir() && name != "obj" && name != "bin" {
            copy_tree(&path, &to.join(&name));
        } else if path.is_file() {
            std::fs::copy(&path, to.join(&name)).unwrap();
        }
    }
}

/// The C# harness with a Cargo package and a Python project beside it.
pub(super) fn workspace(base: &Path) -> PathBuf {
    let root: PathBuf = base.canonicalize().unwrap().join("ws");
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/_Harness"),
        &root,
    );
    let files: [(&str, &str); 4] = [
        ("Cargo.toml", CARGO),
        ("src/main.rs", RS_TEXT),
        ("pyproject.toml", PYPROJECT),
        (PY, PY_TEXT),
    ];
    for (file, text) in files {
        let path: PathBuf = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
    }
    root
}

pub(super) fn installed(server: Server) -> bool {
    let found: bool = match server {
        Server::Roslyn => bb_lsp::roslyn::locate().is_ok(),
        Server::RustAnalyzer => {
            bb_cargo::analyzer::locate(&bb_cargo::analyzer::Inputs::from_env()).is_ok()
        }
        Server::Pyright => {
            bb_python::pyright::locate(&bb_python::pyright::Inputs::from_env()).is_ok()
        }
    };
    assert!(
        found || !tools_required(),
        "BACKBACK_REQUIRE_TOOLS is set but no {} runs",
        server.name()
    );
    found
}

/// Each probed file reached its own server exactly once, with `texts`.
fn opened_once(live: &Live, log: &Shared, texts: &[String]) {
    let started: Instant = Instant::now();
    while log.lock().unwrap().opens.len() < PROBES.len() && started.elapsed().as_secs() < 30 {
        std::thread::sleep(Duration::from_millis(50));
    }
    let opens: Vec<(Server, PathBuf, String)> = log
        .lock()
        .unwrap()
        .opens
        .iter()
        .map(|(_, s, p, t): &(usize, Server, PathBuf, String)| (*s, p.clone(), t.clone()))
        .collect();
    for (probe, text) in PROBES.iter().zip(texts) {
        let path: PathBuf = live.root.join(probe.file);
        let sent: Vec<&(Server, PathBuf, String)> = opens
            .iter()
            .filter(|(_, p, _): &&(Server, PathBuf, String)| *p == path)
            .collect();
        assert_eq!(
            sent,
            vec![&(probe.server, path.clone(), text.clone())],
            "one didOpen of {} to its server, with the tab's text",
            probe.file
        );
    }
}

fn answers(live: &mut Live, log: &Shared, probe: &Probe) {
    let target: PathBuf = live.root.join(probe.def.0);
    let def_line: usize = probe.def.1;
    live.until(
        &format!("F12 in {}", probe.file),
        &mut |l: &mut Live| {
            l.open(probe.file);
            l.cursor_after(probe.f12_after);
            l.keys("f12");
        },
        &move |v: &ExplorerView| active(v) == Some((target.clone(), def_line)),
    );
    live.open(probe.file);
    live.until(
        &format!("the hover in {}", probe.file),
        &mut |l: &mut Live| {
            let at: gpui::Point<gpui::Pixels> = l.point(probe.hover_at.0, probe.hover_at.1);
            l.rest_mouse(at);
        },
        &|v: &ExplorerView| {
            v.intel
                .hover
                .as_ref()
                .is_some_and(|h: &crate::intel::model::popups::HoverPopup| {
                    h.text.contains(probe.hover_has)
                })
        },
    );
    live.keys("escape");
    let path: PathBuf = live.root.join(probe.file);
    assert!(
        log.lock().unwrap().diagnosed.contains(&path),
        "{}",
        probe.file
    );
}

/// Pyright's error on the line typed while the walk ran (`zz`, line 5).
fn typed_line_has_its_error(live: &mut Live) {
    let py: PathBuf = live.root.join(PY);
    live.until(
        "the error on the typed line",
        &mut |_: &mut Live| {},
        &|v: &ExplorerView| {
            v.intel
                .problems_in(&v.root)
                .iter()
                .any(|(p, d): &(&Path, &bb_intel::Diagnostic)| {
                    *p == py.as_path()
                        && d.span.start.line == 5
                        && d.severity == Some(bb_intel::Severity::Error)
                })
        },
    );
}

#[gpui::test]
fn files_opened_while_detecting_reach_the_real_servers_once_with_their_typed_text(
    cx: &mut TestAppContext,
) {
    let scratch: Scratch = Scratch::written(&mut |base: &Path| workspace(base));
    let picked: bb_lang::LangSet = bb_lang::LangSet::of(&bb_lang::Lang::ALL);
    crate::launch::recent::save_langs(&scratch.root, picked);
    let (log, bridges): (Shared, Rc<RefCell<Vec<Bridge>>>) = (Shared::default(), Rc::default());
    let _starts: Starts = Starts::install(starter(&log, &bridges));
    let slow: SlowWalks = SlowWalks::hold();
    let window: WindowHandle<ExplorerView> = launch(cx, &scratch.root);
    assert!(detecting(window, cx), "the window waited for the walk");
    let vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    let mut live: Live = Live::new(scratch.root.clone(), window, vcx);
    live.bridges = Rc::clone(&bridges);
    for probe in &PROBES {
        live.open(probe.file);
    }
    live.cursor_after("add(1, 2)");
    live.keys("enter z z");
    let texts: Vec<String> = PROBES
        .iter()
        .map(|p: &Probe| {
            let path: PathBuf = live.root.join(p.file);
            live.tab_text(&path).unwrap()
        })
        .collect();
    assert!(texts[2].ends_with("add(1, 2)\nzz\n"), "{:?}", texts[2]);
    assert!(
        log.lock().unwrap().opens.is_empty(),
        "no server before the answer"
    );
    drop(slow);
    wait_for_languages(window, cx);
    opened_once(&live, &log, &texts);
    for probe in PROBES.iter().filter(|p: &&Probe| installed(p.server)) {
        answers(&mut live, &log, probe);
    }
    if installed(Server::Pyright) {
        typed_line_has_its_error(&mut live);
    }
    opened_once(&live, &log, &texts);
    assert_eq!(log.lock().unwrap().sync_failed, Vec::<String>::new());
    super::close_window(window, cx);
    super::relay::wait_ended(&log);
}
