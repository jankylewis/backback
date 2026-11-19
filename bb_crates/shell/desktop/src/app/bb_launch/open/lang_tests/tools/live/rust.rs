//! BBEOD29: a Rust window on the real rust-analyzer, by mouse and keys: F12
//! and Cmd+click open `add` in `math.rs`, hover shows `fn add`, F2 renames
//! it in both files, Problems lists beta's compile error and a click opens
//! its line; no `target/` appears in the workspace. Without rust-analyzer
//! the status shows its install hint (a failure when
//! `BACKBACK_REQUIRE_TOOLS`).

use std::path::PathBuf;

use bb_cargo::analyzer::{self, Inputs, MISSING_HINT};
use bb_cargo::fixtures::{Shape, ADD_LINE, BROKEN_LINE, CALL_LINE};
use gpui::{Modifiers, TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::close_all;
use super::super::rust::panel_kit::{rust_window, workspace};
use super::bb_kit::{active, listing, tools_required, Live};
use crate::ExplorerView;

const LIB: &str = "alpha/src/lib.rs";
const MATH: &str = "alpha/src/math.rs";
const BETA: &str = "beta/src/lib.rs";
/// The 0-based column inside `add` in `math::add(1, 2)`.
const IN_ADD: u32 = 11;

fn goes_to_add(live: &mut Live) {
    let math: PathBuf = live.root.join(MATH);
    let add_line: usize = usize::try_from(ADD_LINE - 1).unwrap();
    let at_add: &dyn Fn(&ExplorerView) -> bool = &move |v: &ExplorerView| {
        active(v).is_some_and(|(p, line): (PathBuf, usize)| p == math && line == add_line)
    };
    live.until(
        "F12 to add",
        &mut |l: &mut Live| {
            l.open(LIB);
            l.cursor_after("math::ad");
            l.keys("f12");
        },
        at_add,
    );
    live.until(
        "Cmd+click to add",
        &mut |l: &mut Live| {
            l.open(LIB);
            let at: gpui::Point<gpui::Pixels> = l.point(CALL_LINE - 1, IN_ADD);
            l.click_at(at, Modifiers::command());
        },
        at_add,
    );
}

fn hovers_and_renames(live: &mut Live) {
    live.open(LIB);
    live.until(
        "the hover on add",
        &mut |l: &mut Live| {
            let at: gpui::Point<gpui::Pixels> = l.point(CALL_LINE - 1, IN_ADD);
            l.rest_mouse(at);
        },
        &|v: &ExplorerView| {
            v.intel
                .hover
                .as_ref()
                .is_some_and(|h: &crate::intel::model::popups::HoverPopup| {
                    h.text.contains("fn add")
                })
        },
    );
    live.until(
        "the rename preview",
        &mut |l: &mut Live| {
            l.keys("escape");
            l.cursor_after("math::ad");
            l.keys("f2 backspace backspace backspace p l u s enter");
        },
        &|v: &ExplorerView| v.intel.rename_preview.is_some(),
    );
    live.keys("enter");
    let lib: PathBuf = live.root.join(LIB);
    let math: PathBuf = live.root.join(MATH);
    let (lib_text, math_text): (String, String) =
        (live.tab_text(&lib).unwrap(), live.tab_text(&math).unwrap());
    assert!(lib_text.contains("math::plus(1, 2)"), "{lib_text}");
    assert!(math_text.contains("pub fn plus("), "{math_text}");
}

fn problems_open_the_error(live: &mut Live) {
    let beta: PathBuf = live.root.join(BETA);
    let error: PathBuf = beta.clone();
    live.open(BETA);
    live.until(
        "beta's compile error in Problems",
        &mut |_: &mut Live| {},
        &move |v: &ExplorerView| {
            v.intel.problems_in(&v.root).iter().any(
                |(p, d): &(&std::path::Path, &bb_intel::Diagnostic)| {
                    *p == error.as_path() && d.span.start.line == BROKEN_LINE - 1
                },
            )
        },
    );
    live.open(LIB);
    let editor: gpui::Point<gpui::Pixels> = live.center_of("editor-area");
    live.click_at(editor, Modifiers::default());
    live.keys("cmd-shift-m");
    live.vcx.run_until_parked();
    let row: usize = live.error_row(&beta, BROKEN_LINE - 1);
    let first: gpui::Point<gpui::Pixels> = live.center_of(format!("place-{row}").leak());
    live.click_at(first, Modifiers::default());
    live.vcx.run_until_parked();
    let error_line: usize = usize::try_from(BROKEN_LINE - 1).unwrap();
    assert_eq!(live.read(active), Some((beta, error_line)));
}

#[gpui::test]
fn rust_analyzer_by_mouse_and_keys_f12_click_hover_rename_problems_or_its_install_hint(
    cx: &mut TestAppContext,
) {
    let (_base, root): (tempfile::TempDir, PathBuf) = workspace(Shape::Broken);
    let (window, vcx): (WindowHandle<ExplorerView>, VisualTestContext) = rust_window(cx, &root);
    let mut live: Live = Live::new(root, window, vcx);
    live.vcx
        .update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    let before: Vec<PathBuf> = listing(&live.root);
    live.attach(bb_intel::Server::RustAnalyzer);
    if analyzer::locate(&Inputs::from_env()).is_ok() {
        live.until(
            "Rust: Ready",
            &mut |_: &mut Live| {},
            &|v: &ExplorerView| crate::launch::langs::status_line(v).contains("Rust: Ready"),
        );
        goes_to_add(&mut live);
        hovers_and_renames(&mut live);
        problems_open_the_error(&mut live);
    } else {
        assert!(
            !tools_required(),
            "BACKBACK_REQUIRE_TOOLS is set but no rust-analyzer runs"
        );
        live.until(
            "the install hint",
            &mut |_: &mut Live| {},
            &|v: &ExplorerView| crate::launch::langs::status_line(v).ends_with(MISSING_HINT),
        );
    }
    let added: Vec<PathBuf> = listing(&live.root)
        .into_iter()
        .filter(|p: &PathBuf| !before.contains(p))
        .collect();
    assert!(
        added
            .iter()
            .all(|p: &PathBuf| *p == live.root.join("Cargo.lock")),
        "no target/ or other file in the workspace (Cargo.lock: BBEOD29 READY ruling): {added:?}"
    );
    close_all(cx);
}
