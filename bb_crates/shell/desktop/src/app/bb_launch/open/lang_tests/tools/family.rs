//! BBEOD40, C#-only assumptions in language-neutral paths. A file of no
//! language (`Cargo.toml`, a README) has no server: `server_for` sent it to
//! Roslyn, so its open, edits and F12 reached the C# server and a C#-less
//! window said `C# server: …` (BB-BUG-090). Find in Files' Project places
//! were `.csproj` folders only: none in a Rust or Python window (BB-BUG-091).
//! A rename's closed files and Cmd+N opened as C# tabs (BB-BUG-092).

use std::path::PathBuf;

use bb_intel::{DocPath, DocUri, FileChange, Span, TextChange};
use bb_lang::{FileMode, Lang, LangSet};
use bb_text::LspPoint;
use gpui::{TestAppContext, VisualTestContext};

use super::rust::bb_rig::{drain, files, rig_with, RustRig, MATH};
use crate::intel::bb_worker::protocol::{Answer, Ask, Ticket};
use crate::ExplorerView;

const PLAIN: &str = "alpha/Cargo.toml";
const NO_SERVER: &str = "No language server for this file";

fn type_and_ask(rig: &RustRig, vcx: &mut VisualTestContext) -> Option<String> {
    rig.open(vcx, PLAIN);
    vcx.simulate_keystrokes("x");
    vcx.simulate_keystrokes("f12");
    vcx.run_until_parked();
    rig.view
        .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.status.clone())
}

#[gpui::test]
fn a_file_of_no_language_reaches_no_server_in_a_window_of_all_three(cx: &mut TestAppContext) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) =
        rig_with(cx, LangSet::of(&Lang::ALL), true);
    let said: Option<String> = type_and_ask(&rig, vcx);
    for server in [&mut rig.csharp, &mut rig.rust, &mut rig.python] {
        let sent: Vec<(String, PathBuf)> = files(&drain(server));
        assert!(sent.is_empty(), "{sent:?}");
    }
    assert_eq!(said.as_deref(), Some(NO_SERVER));
}

#[gpui::test]
fn f12_in_a_file_of_no_language_in_a_rust_window_never_names_csharp(cx: &mut TestAppContext) {
    let (rig, vcx): (RustRig, &mut VisualTestContext) =
        rig_with(cx, LangSet::of(&[Lang::Rust]), false);
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            v.link = None;
        },
    );
    let said: Option<String> = type_and_ask(&rig, vcx);
    assert!(!said.as_deref().unwrap_or("").contains("C#"), "{said:?}");
    assert_eq!(said.as_deref(), Some(NO_SERVER));
}

#[gpui::test]
fn find_in_files_offers_each_cargo_crate_as_a_project(cx: &mut TestAppContext) {
    let (rig, vcx): (RustRig, &mut VisualTestContext) =
        rig_with(cx, LangSet::of(&[Lang::Rust]), false);
    rig.open(vcx, "alpha/src/lib.rs");
    vcx.dispatch_action(crate::quick_open::FindInFiles);
    vcx.run_until_parked();
    let places: Vec<String> = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.quick_open
            .as_ref()
            .map_or_else(Vec::new, |quick: &crate::quick_open::QuickOpen| {
                quick
                    .matches()
                    .iter()
                    .map(|p: &&std::path::Path| p.display().to_string())
                    .collect()
            })
    });
    for crate_dir in ["alpha", "beta"] {
        let label: String = format!("Project · {crate_dir} · {crate_dir}");
        assert!(places.contains(&label), "{label} in {places:?}");
    }
}

#[gpui::test]
fn a_rust_rename_opens_the_closed_file_it_edits_with_rust_colours(cx: &mut TestAppContext) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) =
        rig_with(cx, LangSet::of(&[Lang::Rust]), false);
    rig.open(vcx, "alpha/src/lib.rs");
    rig.cursor_after(vcx, "pub fn tot");
    vcx.simulate_keystrokes("f2 backspace T o t a l enter");
    let (ticket, ask): (Ticket, Ask) = rig.last_rust_ask();
    assert!(matches!(ask, Ask::Rename(..)), "{ask:?}");
    let math: PathBuf = rig.root.join(MATH);
    let start: LspPoint = LspPoint {
        line: 0,
        character: 0,
    };
    let change: TextChange = TextChange {
        span: Span { start, end: start },
        new_text: String::from("// Total\n"),
    };
    let plan: Answer = Answer::Rename(vec![FileChange::Edit {
        uri: DocUri::File(DocPath::new(math.clone()).unwrap()),
        changes: vec![change],
    }]);
    rig.reply(vcx, ticket, plan);
    vcx.simulate_keystrokes("enter");
    vcx.run_until_parked();
    let mode: Option<FileMode> = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs
            .items()
            .iter()
            .find(|tab: &&crate::editor::Tab| tab.path == math)
            .map(|tab: &crate::editor::Tab| tab.mode)
    });
    assert_eq!(mode, Some(FileMode::Colours(Lang::Rust)));
    assert!(std::fs::read_to_string(&math)
        .unwrap()
        .starts_with("// Total\n"));
}

#[gpui::test]
fn cmd_n_in_a_rust_window_opens_a_plain_new_file_not_a_csharp_one(cx: &mut TestAppContext) {
    let (rig, vcx): (RustRig, &mut VisualTestContext) =
        rig_with(cx, LangSet::of(&[Lang::Rust]), false);
    rig.open(vcx, "alpha/src/lib.rs");
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    vcx.simulate_keystrokes("cmd-n");
    vcx.run_until_parked();
    let (untitled, mode): (bool, FileMode) =
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
            (tab.is_untitled(), tab.mode)
        });
    assert!(untitled);
    assert_eq!(mode, FileMode::Plain);
}
