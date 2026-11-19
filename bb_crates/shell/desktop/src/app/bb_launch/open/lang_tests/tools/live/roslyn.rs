//! BB-LSP-003: a C# window on the REAL Roslyn, by keys. Typing `zzz` adds
//! problems; three backspaces must bring back the loaded count. Each second
//! after the deletion is logged (`lsp003`): shown version, the version of
//! the last list the server sent, and the problem count.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bb_lang::{Lang, LangSet};
use gpui::{TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::{close_all, open_with};
use super::bb_kit::Live;
use crate::intel::bb_worker::protocol::Event;
use crate::ExplorerView;

const SPEC: &str = "_NUnitApex/GreeterSpec.cs";
const TYPED_AFTER: &str = "new Greeter();";
const SETTLE: Duration = Duration::from_secs(30);

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

fn harness_window(cx: &mut TestAppContext) -> (tempfile::TempDir, Live) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = base.path().canonicalize().unwrap().join("ws");
    let fixture: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/_Harness");
    copy_tree(&fixture, &root);
    let window: WindowHandle<ExplorerView> = open_with(cx, &root, LangSet::of(&[Lang::CSharp]));
    let vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    (base, Live::new(root, window, vcx))
}

/// Move the server's events into the window, keeping the version and
/// length of every list sent for `spec`.
fn pump_logged(live: &mut Live, spec: &Path, sent: &mut Vec<(i32, usize)>) {
    if let Some((from, to)) = live.bridge.as_mut() {
        while let Ok(event) = from.try_recv() {
            if let Event::Diagnostics {
                path,
                version,
                list,
            } = &event
            {
                if path == spec {
                    sent.push((version.raw(), list.len()));
                }
            }
            to.send(event).unwrap();
        }
    }
    live.vcx.run_until_parked();
}

fn count(live: &mut Live, spec: &Path) -> (Option<i32>, usize) {
    live.read(|v: &ExplorerView| {
        let shown: Option<i32> = v
            .intel
            .shown_version(spec)
            .map(|d: bb_intel::DocVersion| d.raw());
        (shown, v.intel.diagnostics_of(spec).len())
    })
}

/// Wait until the count of the shown version stays the same for 3 s.
fn settled(live: &mut Live, spec: &Path, sent: &mut Vec<(i32, usize)>) -> usize {
    let started: Instant = Instant::now();
    let mut last: (Option<i32>, usize) = (None, usize::MAX);
    let mut since: Instant = Instant::now();
    loop {
        pump_logged(live, spec, sent);
        let now: (Option<i32>, usize) = count(live, spec);
        let answered: bool = sent
            .last()
            .is_some_and(|(v, _): &(i32, usize)| Some(*v) == now.0);
        if now != last {
            last = now;
            since = Instant::now();
        } else if answered && since.elapsed() > Duration::from_secs(3) {
            return now.1;
        }
        assert!(started.elapsed() < Duration::from_secs(240), "{sent:?}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[gpui::test]
fn roslyn_problems_return_to_the_loaded_count_after_typed_text_is_deleted(cx: &mut TestAppContext) {
    let (_base, mut live): (tempfile::TempDir, Live) = harness_window(cx);
    live.vcx
        .update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    live.attach(bb_intel::Server::Roslyn);
    live.until("C#: Ready", &mut |_: &mut Live| {}, &|v: &ExplorerView| {
        crate::launch::langs::status_line(v).contains("Ready")
    });
    let spec: PathBuf = live.root.join(SPEC);
    let original: String = std::fs::read_to_string(&spec).unwrap();
    let mut sent: Vec<(i32, usize)> = Vec::new();
    live.open(SPEC);
    let loaded: usize = settled(&mut live, &spec, &mut sent);
    live.cursor_after(TYPED_AFTER);
    live.keys("z z z");
    let typed: usize = settled(&mut live, &spec, &mut sent);
    assert!(typed > loaded, "typed {typed} loaded {loaded} {sent:?}");
    live.keys("backspace backspace backspace");
    assert_eq!(live.tab_text(&spec), Some(original));
    let deleted: Instant = Instant::now();
    let mut log: Vec<String> = Vec::new();
    let mut back: Option<Duration> = None;
    while deleted.elapsed() < SETTLE {
        pump_logged(&mut live, &spec, &mut sent);
        let (shown, problems): (Option<i32>, usize) = count(&mut live, &spec);
        log.push(format!(
            "lsp003 t={:.1}s shown={shown:?} last_sent={:?} problems={problems}",
            deleted.elapsed().as_secs_f32(),
            sent.last()
        ));
        if problems == loaded && back.is_none() {
            back = Some(deleted.elapsed());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    eprintln!(
        "lsp003 loaded={loaded} typed={typed} back={back:?}\n{}",
        log.join("\n")
    );
    let (_, last): (Option<i32>, usize) = count(&mut live, &spec);
    assert_eq!(last, loaded, "{log:#?}");
    assert!(
        back.is_some_and(|b: Duration| b < Duration::from_secs(5)),
        "{log:#?}"
    );
    close_all(cx);
}
