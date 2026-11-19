//! BBEOD29: a Python window on the real pyright, by mouse and keys: F12 and
//! Cmd+click open `add` in its module, hover shows its signature, F2
//! renames it in both files, Problems lists the syntax error and a click
//! opens its line; nothing is written into the folder. Without pyright the
//! status shows its install hint (a failure when `BACKBACK_REQUIRE_TOOLS`).

use std::path::PathBuf;

use bb_lang::{Lang, LangSet};
use bb_python::fixtures::{
    self, Shape, ADD_LINE, BROKEN_FILE, BROKEN_LINE, CALL_COLUMN, CALL_LINE,
};
use bb_python::pyright::{self, Inputs, MISSING_HINT};
use gpui::{Modifiers, TestAppContext, VisualTestContext, WindowHandle};

use super::super::super::{close_all, open_with};
use super::bb_kit::{active, listing, tools_required, Live};
use crate::ExplorerView;

fn python_window(cx: &mut TestAppContext) -> (tempfile::TempDir, Live) {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let root: PathBuf = base.path().canonicalize().unwrap().join("py");
    fixtures::write(&root, Shape::Broken).unwrap();
    let window: WindowHandle<ExplorerView> = open_with(cx, &root, LangSet::of(&[Lang::Python]));
    let vcx: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    (base, Live::new(root, window, vcx))
}

fn goes_to_add(live: &mut Live) {
    let math: PathBuf = live.root.join(fixtures::MATH_FILE);
    let add_line: usize = usize::try_from(ADD_LINE - 1).unwrap();
    let at_add: &dyn Fn(&ExplorerView) -> bool = &move |v: &ExplorerView| {
        active(v).is_some_and(|(p, line): (PathBuf, usize)| p == math && line == add_line)
    };
    live.open(fixtures::CART_FILE);
    live.until(
        "F12 to add",
        &mut |l: &mut Live| {
            l.open(fixtures::CART_FILE);
            l.cursor_after("return ad");
            l.keys("f12");
        },
        at_add,
    );
    live.until(
        "Cmd+click to add",
        &mut |l: &mut Live| {
            l.open(fixtures::CART_FILE);
            let at: gpui::Point<gpui::Pixels> = l.point(CALL_LINE - 1, CALL_COLUMN + 1);
            l.click_at(at, Modifiers::command());
        },
        at_add,
    );
}

fn hovers_and_renames(live: &mut Live) {
    live.open(fixtures::CART_FILE);
    live.until(
        "the hover on add",
        &mut |l: &mut Live| {
            let at: gpui::Point<gpui::Pixels> = l.point(CALL_LINE - 1, CALL_COLUMN + 1);
            l.rest_mouse(at);
        },
        &|v: &ExplorerView| {
            v.intel
                .hover
                .as_ref()
                .is_some_and(|h: &crate::intel::model::popups::HoverPopup| h.text.contains("add"))
        },
    );
    live.until(
        "the rename preview",
        &mut |l: &mut Live| {
            l.keys("escape");
            l.cursor_after("return ad");
            l.keys("f2 backspace backspace backspace p l u s enter");
        },
        &|v: &ExplorerView| v.intel.rename_preview.is_some(),
    );
    live.keys("enter");
    let cart: PathBuf = live.root.join(fixtures::CART_FILE);
    let math: PathBuf = live.root.join(fixtures::MATH_FILE);
    let (cart_text, math_text): (String, String) =
        (live.tab_text(&cart).unwrap(), live.tab_text(&math).unwrap());
    let said: Option<String> = live.read(|v: &ExplorerView| v.status.clone());
    assert!(
        cart_text.contains("return plus(1, 2)"),
        "{said:?} {cart_text}"
    );
    assert!(math_text.contains("def plus("), "{math_text}");
}

fn problems_open_the_error(live: &mut Live) {
    let broken: PathBuf = live.root.join(BROKEN_FILE);
    let error: PathBuf = broken.clone();
    live.open(BROKEN_FILE);
    live.until(
        "the syntax error in Problems",
        &mut |_: &mut Live| {},
        &move |v: &ExplorerView| {
            v.intel.problems_in(&v.root).iter().any(
                |(p, d): &(&std::path::Path, &bb_intel::Diagnostic)| {
                    *p == error.as_path() && d.span.start.line == BROKEN_LINE - 1
                },
            )
        },
    );
    live.open(fixtures::CART_FILE);
    let editor: gpui::Point<gpui::Pixels> = live.center_of("editor-area");
    live.click_at(editor, Modifiers::default());
    live.keys("cmd-shift-m");
    live.vcx.run_until_parked();
    let row: usize = live.error_row(&broken, BROKEN_LINE - 1);
    let first: gpui::Point<gpui::Pixels> = live.center_of(format!("place-{row}").leak());
    live.click_at(first, Modifiers::default());
    live.vcx.run_until_parked();
    assert_eq!(
        live.read(active),
        Some((broken, usize::try_from(BROKEN_LINE - 1).unwrap()))
    );
}

#[gpui::test]
fn pyright_by_mouse_and_keys_f12_click_hover_rename_problems_or_its_install_hint(
    cx: &mut TestAppContext,
) {
    let (_base, mut live): (tempfile::TempDir, Live) = python_window(cx);
    live.vcx
        .update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    let before: Vec<PathBuf> = listing(&live.root);
    live.attach(bb_intel::Server::Pyright);
    if pyright::locate(&Inputs::from_env()).is_ok() {
        live.until(
            "Python: Ready",
            &mut |_: &mut Live| {},
            &|v: &ExplorerView| crate::launch::langs::status_line(v).contains("Python: Ready"),
        );
        goes_to_add(&mut live);
        hovers_and_renames(&mut live);
        problems_open_the_error(&mut live);
    } else {
        assert!(
            !tools_required(),
            "BACKBACK_REQUIRE_TOOLS is set but no pyright runs"
        );
        live.until(
            "the install hint",
            &mut |_: &mut Live| {},
            &|v: &ExplorerView| crate::launch::langs::status_line(v).ends_with(MISSING_HINT),
        );
    }
    assert_eq!(
        listing(&live.root),
        before,
        "nothing written into the folder"
    );
    close_all(cx);
}
