//! BBEOD39 (b): the walk's answer lands while a big file's background parse
//! still runs. The tab keeps its plain mode at the answer (the parse holds
//! its parser); when the parse ends the tab is Rust-coloured, its tree
//! holds the key typed before the answer, and nothing panics.

use std::fmt::Write;
use std::path::{Path, PathBuf};

use bb_lang::{FileMode, Lang, LangSet, Langs};
use gpui::{App, Keystroke, TestAppContext, WindowHandle};

use super::{detecting, launch, tab_mode, Scratch, SlowWalks};
use crate::launch::WindowLangs;
use crate::ExplorerView;

const HEAD: &str = "// big";

/// A Cargo package whose `src/main.rs` has 3,000 functions (a test walk is held 3 s at most).
fn big_package(base: &Path) -> PathBuf {
    let root: PathBuf = base.canonicalize().unwrap().join("big");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"big\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let mut text: String = format!("{HEAD}\nfn main() {{}}\n");
    for n in 0..3_000 {
        writeln!(text, "fn f{n}() -> i32 {{ {n} }}").unwrap();
    }
    std::fs::write(root.join("src/main.rs"), text).unwrap();
    root
}

/// Whether the tab's parse runs, its mode, and its tree's root kind, first
/// node's end and end byte, with its text's length.
type Seen = (bool, FileMode, String, usize, usize, usize);

fn seen(window: WindowHandle<ExplorerView>, cx: &mut TestAppContext) -> Seen {
    window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
            let root: tree_sitter::Node<'_> = tab.live.tree().root_node();
            let first: usize = root
                .child(0)
                .map_or(0, |n: tree_sitter::Node<'_>| n.end_byte());
            let text: String = tab.document.text();
            (
                tab.live.parsing(),
                tab.mode,
                root.kind().to_owned(),
                first,
                root.end_byte(),
                text.len(),
            )
        })
        .unwrap()
}

/// Type `z` after the head comment, without a frame or a parked executor:
/// its background parse is left running.
fn type_z_after_head(window: WindowHandle<ExplorerView>, cx: &mut TestAppContext) {
    window
        .update(
            cx,
            |v: &mut ExplorerView, _: &mut gpui::Window, _: &mut gpui::Context<ExplorerView>| {
                let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
                doc.set_cursor(bb_text::CharIdx::new(HEAD.len())).unwrap();
            },
        )
        .unwrap();
    cx.dispatch_keystroke(window.into(), Keystroke::parse("z").unwrap());
    assert!(seen(window, cx).0, "the key started a background parse");
}

#[gpui::test]
fn a_big_file_still_parsing_when_the_languages_land_ends_coloured_with_its_edit(
    cx: &mut TestAppContext,
) {
    let scratch: Scratch = Scratch::written(&mut |base: &Path| big_package(base));
    let slow: SlowWalks = SlowWalks::hold();
    let window: WindowHandle<ExplorerView> = launch(cx, &scratch.root.join("src/main.rs"));
    assert!(detecting(window, cx), "the window waited for the walk");
    assert_eq!(tab_mode(window, cx), Some(FileMode::Plain));
    type_z_after_head(window, cx);
    let rust: LangSet = LangSet::of(&[Lang::Rust]);
    window
        .update(
            cx,
            |v: &mut ExplorerView, w: &mut gpui::Window, cx: &mut gpui::Context<ExplorerView>| {
                v.langs = WindowLangs::new(Langs::new(rust, rust));
                crate::launch::languages_found(v, w, cx);
            },
        )
        .unwrap();
    let (parsing, mode, ..): Seen = seen(window, cx);
    assert_eq!(
        (parsing, mode),
        (true, FileMode::Plain),
        "the answer landed mid-parse"
    );
    cx.run_until_parked();
    let after: Seen = seen(window, cx);
    let typed: usize = HEAD.len() + 1;
    let want: Seen = (
        false,
        FileMode::Colours(Lang::Rust),
        String::from("source_file"),
        typed,
        after.5,
        after.5,
    );
    assert_eq!(after, want, "coloured, with the typed key in its tree");
    let text: Option<String> = window
        .read_with(cx, |v: &ExplorerView, _: &App| {
            v.tabs
                .active_tab()
                .map(|t: &crate::editor::Tab| t.document.text())
        })
        .unwrap();
    assert!(text.is_some_and(|t: String| t.starts_with("// bigz\n")));
    super::close_window(window, cx);
    drop(slow);
}

/// BB-GAT-018: CI opened the big file in 3.09 s under load and the held
/// walk, capped at 3 s, answered first; a hold lasts until it is released.
#[test]
fn a_held_walk_still_has_no_answer_after_the_old_three_second_cap() {
    let scratch: Scratch = Scratch::of("py_only");
    let slow: SlowWalks = SlowWalks::hold();
    crate::launch::langs::walk_early(&scratch.root, None);
    std::thread::sleep(std::time::Duration::from_millis(3_500));
    let resolved: crate::launch::langs::Resolved =
        crate::launch::langs::resolve(&scratch.root, None);
    assert!(
        matches!(resolved, crate::launch::langs::Resolved::Later(_)),
        "the walk answered while held"
    );
    drop(slow);
}
