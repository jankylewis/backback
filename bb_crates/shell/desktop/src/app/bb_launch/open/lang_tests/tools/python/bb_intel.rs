//! BBEOD28 in a real window on the `all3` fixture with three fake server
//! threads: a `.py`, a `.rs` and a `.cs` tab each reach only their own
//! server (none of the others reach Roslyn); F12 in a `.py` file asks
//! pyright and opens the definition in another module; a missing pyright
//! shows its install hint while C# keeps working.

use std::path::PathBuf;

use bb_lang::{FileMode, Lang, LangSet};
use bb_python::fixtures::{ADD_LINE, CALL_LINE, CART_FILE, MATH_FILE};
use bb_python::pyright::MISSING_HINT;
use gpui::{TestAppContext, VisualTestContext};

use super::super::rust::bb_rig::{drain, files, kinds, reply_on, rig_with, RustRig, CS, LIB};
use crate::intel::bb_worker::protocol::{Answer, Ask, Event, Ticket};
use crate::intel::glue::window_tests::bb_rig::place;
use crate::ExplorerView;

#[gpui::test]
fn in_all3_each_tab_reaches_only_its_own_server(cx: &mut TestAppContext) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) =
        rig_with(cx, LangSet::of(&Lang::ALL), true);
    for file in [CART_FILE, LIB, CS] {
        rig.open(vcx, file);
        vcx.simulate_keystrokes("x");
        vcx.simulate_keystrokes("f12");
        vcx.run_until_parked();
    }
    let py: PathBuf = rig.root.join(CART_FILE);
    let rs: PathBuf = rig.root.join(LIB);
    let cs: PathBuf = rig.root.join(CS);
    let sent: [(Vec<(String, PathBuf)>, &PathBuf); 3] = [
        (files(&drain(&mut rig.python)), &py),
        (files(&drain(&mut rig.rust)), &rs),
        (files(&drain(&mut rig.csharp)), &cs),
    ];
    for (to_server, own) in &sent {
        assert_eq!(
            kinds(to_server, own),
            ["open", "change", "ask"],
            "{to_server:?}"
        );
        assert!(
            to_server.iter().all(|(_, p): &(String, PathBuf)| p == *own),
            "{to_server:?}"
        );
    }
}

#[gpui::test]
fn f12_in_a_python_file_asks_pyright_and_opens_the_definition_in_another_module(
    cx: &mut TestAppContext,
) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) =
        rig_with(cx, LangSet::of(&[Lang::Python]), true);
    rig.open(vcx, CART_FILE);
    rig.cursor_after(vcx, "return ad");
    vcx.simulate_keystrokes("f12");
    let (ticket, ask): (Ticket, Ask) = rig.last_python_ask();
    assert!(
        matches!(ask, Ask::Definition(p) if p.line == CALL_LINE - 1),
        "{ask:?}"
    );
    let math: PathBuf = rig.root.join(MATH_FILE);
    let answer: Answer = Answer::Definition(vec![place(&math, ADD_LINE - 1)]);
    reply_on(&rig.python_events, vcx, (ticket, answer));
    assert_eq!(rig.active(vcx), (math, 0));
    assert!(drain(&mut rig.csharp).is_empty(), "nothing reached Roslyn");
}

#[gpui::test]
fn a_missing_pyright_shows_its_hint_and_csharp_and_python_colours_keep_working(
    cx: &mut TestAppContext,
) {
    let (mut rig, vcx): (RustRig, &mut VisualTestContext) =
        rig_with(cx, LangSet::of(&[Lang::CSharp, Lang::Python]), true);
    rig.python_events
        .send(Event::Unavailable(MISSING_HINT.to_owned()))
        .unwrap();
    vcx.run_until_parked();
    rig.open(vcx, CART_FILE);
    vcx.simulate_keystrokes("f12");
    let (line, said, mode): (String, Option<String>, FileMode) =
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            (
                crate::launch::langs::status_line(v),
                v.status.clone(),
                v.tabs.active_tab().unwrap().mode,
            )
        });
    assert_eq!(line, format!("C# · Python · Ready · {MISSING_HINT}"));
    assert_eq!(said, Some(format!("Python server: {MISSING_HINT}")));
    assert_eq!(mode, FileMode::Colours(Lang::Python));
    assert!(
        drain(&mut rig.python).is_empty(),
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
