//! Completion keys: `.`, Ctrl+Space, filter, Enter, stale answers.

use bb_text::LspPoint;

use super::bb_rig::items;
use crate::intel::bb_worker::protocol::{Ask, Request, Ticket};
use gpui::{TestAppContext, VisualTestContext};

use super::bb_rig::{rig, Rig};
use crate::ExplorerView;

#[gpui::test]
fn typing_a_dot_syncs_then_asks_and_enter_accepts_the_filtered_item(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    rig.cursor_after(vcx, "{ x");
    vcx.simulate_keystrokes(".");
    let sent: Vec<Request> = rig.drain();
    assert!(
        matches!(sent.first(), Some(Request::Change { edits, .. }) if edits[0].text == "."),
        "{sent:?}"
    );
    let Some(Request::Ask {
        ticket,
        ask: Ask::Completion(at),
        ..
    }) = sent.get(1).cloned()
    else {
        panic!("{sent:?}");
    };
    assert_eq!(
        at,
        LspPoint {
            line: 0,
            character: 23
        }
    );
    rig.reply(vcx, ticket, items(&["ToString", "Hello", "Help"]));
    vcx.simulate_keystrokes("h e down enter");
    assert_eq!(rig.text(vcx), "class A { void M() { x.Help } }\n");
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert!(v.intel.completion.is_none());
        let row: &crate::intel::model::TraceRow = v.intel.trace.last().expect("a traced popup");
        assert_eq!(row.frames, 0, "drawn in the first frame after the answer");
    });
}

#[gpui::test]
fn ctrl_space_answer_is_dropped_once_the_cursor_left_the_word(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    rig.cursor_after(vcx, "{ x");
    vcx.simulate_keystrokes("ctrl-space");
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert_eq!(
        ask,
        Ask::Completion(LspPoint {
            line: 0,
            character: 22
        })
    );
    vcx.simulate_keystrokes("space y z");
    assert_eq!(
        rig.text(vcx),
        "class A { void M() { x yz } }\n",
        "keys run while the server thinks"
    );
    rig.reply(vcx, ticket, items(&["xyz"]));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert!(v.intel.completion.is_none());
    });
}
