//! BBEOD27 in a real window with two fake server threads: every request
//! about a Rust file reaches rust-analyzer only, a C# file Roslyn only, a
//! Python file neither (BB-BUG-071); F12 and Cmd+click in a `.rs` file go
//! to the definition; a missing rust-analyzer shows its install hint while
//! C# and the Rust colours keep working.

use std::path::PathBuf;

use bb_cargo::analyzer::MISSING_HINT;
use bb_cargo::fixtures::{ADD_LINE, CALL_LINE};
use bb_lang::{FileMode, Lang, LangSet};
use gpui::{Modifiers, TestAppContext, VisualTestContext};

use super::bb_rig::{drain, files, kinds, rust_rig, RustRig, CS, LIB, MATH, PY};
use crate::intel::bb_worker::protocol::{Answer, Ask, Event, Ticket};
use crate::intel::glue::window_tests::bb_rig::place;
use crate::ExplorerView;

#[gpui::test]
fn every_request_reaches_only_the_server_of_its_file_s_language(cx: &mut TestAppContext) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) = rust_rig(cx, LangSet::of(&Lang::ALL));
    for file in [LIB, CS, PY] {
        rig.open(vcx, file);
        vcx.simulate_keystrokes("x");
        vcx.simulate_keystrokes("f12");
        vcx.run_until_parked();
    }
    let lib: PathBuf = rig.root.join(LIB);
    let cs: PathBuf = rig.root.join(CS);
    let to_rust: Vec<(String, PathBuf)> = files(&drain(&mut rig.rust));
    let to_csharp: Vec<(String, PathBuf)> = files(&drain(&mut rig.csharp));
    assert_eq!(
        kinds(&to_rust, &lib),
        ["open", "change", "ask"],
        "{to_rust:?}"
    );
    assert_eq!(
        kinds(&to_csharp, &cs),
        ["open", "change", "ask"],
        "{to_csharp:?}"
    );
    assert!(
        to_rust.iter().all(|(_, p): &(String, PathBuf)| *p == lib),
        "{to_rust:?}"
    );
    assert!(
        to_csharp.iter().all(|(_, p): &(String, PathBuf)| *p == cs),
        "{to_csharp:?}"
    );
}

#[gpui::test]
fn f12_in_a_rust_file_asks_rust_analyzer_and_opens_the_definition_in_another_file(
    cx: &mut TestAppContext,
) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) =
        rust_rig(cx, LangSet::of(&[Lang::Rust]));
    rig.open(vcx, LIB);
    rig.cursor_after(vcx, "math::a");
    vcx.simulate_keystrokes("f12");
    let (ticket, ask): (Ticket, Ask) = rig.last_rust_ask();
    assert!(
        matches!(ask, Ask::Definition(p) if p.line == CALL_LINE - 1),
        "{ask:?}"
    );
    let math: PathBuf = rig.root.join(MATH);
    rig.reply(
        vcx,
        ticket,
        Answer::Definition(vec![place(&math, ADD_LINE - 1)]),
    );
    assert_eq!(rig.active(vcx), (math, 1));
}

#[gpui::test]
fn cmd_click_on_a_rust_call_goes_to_its_definition(cx: &mut TestAppContext) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) =
        rust_rig(cx, LangSet::of(&[Lang::Rust]));
    rig.open(vcx, LIB);
    let row: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("editor-line-4").unwrap();
    let x: f32 = f32::from(row.origin.x) + 56.0 + 7.8 * 11.0 + 1.0;
    vcx.simulate_click(
        gpui::point(gpui::px(x), row.center().y),
        Modifiers::command(),
    );
    let (ticket, ask): (Ticket, Ask) = rig.last_rust_ask();
    assert!(
        matches!(ask, Ask::Definition(p) if p.line == CALL_LINE - 1),
        "{ask:?}"
    );
    let math: PathBuf = rig.root.join(MATH);
    rig.reply(
        vcx,
        ticket,
        Answer::Definition(vec![place(&math, ADD_LINE - 1)]),
    );
    assert_eq!(rig.active(vcx), (math, 1));
}

#[gpui::test]
fn a_missing_rust_analyzer_shows_its_hint_and_csharp_and_rust_colours_keep_working(
    cx: &mut TestAppContext,
) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) = rust_rig(cx, LangSet::of(&Lang::ALL));
    rig.rust_events
        .send(Event::Unavailable(MISSING_HINT.to_owned()))
        .unwrap();
    vcx.run_until_parked();
    rig.open(vcx, LIB);
    vcx.simulate_keystrokes("f12");
    let (line, said, mode): (String, Option<String>, FileMode) =
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            (
                crate::launch::langs::status_line(v),
                v.status.clone(),
                v.tabs.active_tab().unwrap().mode,
            )
        });
    assert!(line.ends_with(MISSING_HINT), "{line}");
    assert!(line.starts_with("C# · Rust · Python · Ready · "), "{line}");
    assert_eq!(said, Some(format!("Rust server: {MISSING_HINT}")));
    assert_eq!(mode, FileMode::Colours(Lang::Rust));
    assert!(
        drain(&mut rig.rust).is_empty(),
        "nothing is sent to a missing server"
    );
    rig.open(vcx, CS);
    vcx.simulate_keystrokes("f12");
    let to_csharp: Vec<(String, PathBuf)> = files(&drain(&mut rig.csharp));
    assert!(
        to_csharp.contains(&(String::from("ask"), rig.root.join(CS))),
        "{to_csharp:?}"
    );
}
