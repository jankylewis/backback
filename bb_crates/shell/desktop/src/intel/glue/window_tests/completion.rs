//! Completion keys: `.`, Ctrl+Space, filter, Enter, stale answers.

use bb_text::LspPoint;

use super::bb_rig::items;
use crate::intel::bb_worker::protocol::{Ask, AskKind, Event, Request, Ticket};
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

#[gpui::test]
fn stopped_status_fails_questions_and_replays_unsaved_text(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    rig.cursor_after(vcx, "{ x");
    vcx.simulate_keystrokes("ctrl-space");
    rig.last_ask();
    rig.events.send(Event::Stopped).unwrap();
    vcx.run_until_parked();
    rig.view
        .read_with(vcx, |view: &ExplorerView, _: &gpui::App| {
            assert_eq!(view.intel.status.label(), "C# server stopped — Restart");
            assert!(!view.intel.in_flight.waiting(AskKind::Completion));
        });
    vcx.simulate_keystrokes("y");
    let current: String = rig.text(vcx);
    let (sender, mut requests): (
        tokio::sync::mpsc::UnboundedSender<Request>,
        tokio::sync::mpsc::UnboundedReceiver<Request>,
    ) = tokio::sync::mpsc::unbounded_channel();
    let (_events, receiver): (
        tokio::sync::mpsc::UnboundedSender<Event>,
        tokio::sync::mpsc::UnboundedReceiver<Event>,
    ) = tokio::sync::mpsc::unbounded_channel();
    rig.view.update_in(
        vcx,
        |view: &mut ExplorerView,
         window: &mut gpui::Window,
         cx: &mut gpui::Context<ExplorerView>| {
            crate::intel::glue::attach(
                view,
                crate::intel::bb_worker::Link::new(sender),
                receiver,
                window,
                cx,
            );
            super::super::resync_open(view);
        },
    );
    let sent: Request = requests.try_recv().unwrap();
    assert!(matches!(sent, Request::Open { path, text } if path == rig.file && text == current));
}
