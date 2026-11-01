//! F2 rename: preview, apply all or nothing, refusals. Diagnostics: a list
//! shows only on the version it was computed for.

use std::path::PathBuf;

use bb_intel::{Diagnostic, DocVersion, Severity, Span};
use bb_text::LspPoint;

use super::bb_rig::{ask_rename, rename_plan, A};
use crate::intel::bb_worker::protocol::{Event, Request, Ticket};
use gpui::{TestAppContext, VisualTestContext};

use super::bb_rig::{rig, Rig};
use crate::ExplorerView;

#[gpui::test]
fn f2_previews_then_enter_applies_to_the_open_buffer_and_saves_the_closed_file(
    cx: &mut TestAppContext,
) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    let closed: PathBuf = rig.root.join("B.cs");
    std::fs::write(&closed, "var x = x;\n").unwrap();
    let ticket: Ticket = ask_rename(&mut rig, vcx);
    rig.reply(vcx, ticket, rename_plan(&rig.file.clone(), &closed));
    assert_eq!(
        std::fs::read_to_string(&closed).unwrap(),
        "var x = x;\n",
        "preview changes nothing"
    );
    vcx.simulate_keystrokes("enter");
    assert_eq!(
        std::fs::read_to_string(&closed).unwrap(),
        "var Total = Total;\n"
    );
    assert_eq!(
        std::fs::read_to_string(&rig.file).unwrap(),
        A,
        "an open buffer stays unsaved"
    );
    assert_eq!(rig.text(vcx), "class A { void M() { Total } }\n");
    vcx.simulate_keystrokes("cmd-z");
    assert_eq!(rig.text(vcx), A, "one undo step per file");
    let opened: Vec<Request> = rig.drain();
    assert!(
        opened
            .iter()
            .any(|r: &Request| matches!(r, Request::Open { path, .. } if *path == closed)),
        "{opened:?}"
    );
}

#[gpui::test]
fn rename_and_save_are_refused_in_no_write_mode(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, true);
    let closed: PathBuf = rig.root.join("B.cs");
    std::fs::write(&closed, "var x = x;\n").unwrap();
    let ticket: Ticket = ask_rename(&mut rig, vcx);
    rig.reply(vcx, ticket, rename_plan(&rig.file.clone(), &closed));
    vcx.simulate_keystrokes("enter");
    assert_eq!(rig.text(vcx), A);
    assert_eq!(std::fs::read_to_string(&closed).unwrap(), "var x = x;\n");
    vcx.simulate_keystrokes("cmd-s");
    assert_eq!(std::fs::read_to_string(&rig.file).unwrap(), A);
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(v.status.as_deref(), Some("no-write mode: save refused"));
    });
}

#[gpui::test]
fn a_plan_older_than_its_buffer_is_refused_whole(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    let closed: PathBuf = rig.root.join("B.cs");
    std::fs::write(&closed, "var x = x;\n").unwrap();
    let ticket: Ticket = ask_rename(&mut rig, vcx);
    rig.reply(vcx, ticket, rename_plan(&rig.file.clone(), &closed));
    vcx.simulate_keystrokes("w enter");
    assert_eq!(rig.text(vcx), "class A { void M() { wx } }\n");
    assert_eq!(std::fs::read_to_string(&closed).unwrap(), "var x = x;\n");
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let shown: &str = v.status.as_deref().unwrap_or_default();
        assert!(shown.contains("changed since the preview"), "{shown}");
        assert_eq!(v.tabs.items().len(), 1, "the closed file was not opened");
    });
}

fn error_at(character: u32) -> Diagnostic {
    let at: LspPoint = LspPoint { line: 0, character };
    Diagnostic {
        span: Span { start: at, end: at },
        severity: Some(Severity::Error),
        code: None,
        message: format!("at {character}"),
    }
}

fn send(rig: &Rig, version: DocVersion, at: u32) {
    let path: std::path::PathBuf = rig.file.clone();
    let list: Vec<Diagnostic> = vec![error_at(at)];
    rig.events
        .send(Event::Diagnostics {
            path,
            version,
            list,
        })
        .unwrap();
}

/// The squiggle columns the window holds for the rig file.
fn shown_at(rig: &Rig, vcx: &mut VisualTestContext) -> Vec<u32> {
    rig.view
        .read_with(vcx, |view: &ExplorerView, _: &gpui::App| {
            let list: &[Diagnostic] = view.intel.diagnostics_of(&rig.file);
            list.iter()
                .map(|d: &Diagnostic| d.span.start.character)
                .collect()
        })
}

#[gpui::test]
fn a_squiggle_moves_with_the_edit_and_no_old_list_flashes(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    let first: DocVersion = rig.shown(vcx);
    send(&rig, first, 21);
    vcx.run_until_parked();
    assert_eq!(shown_at(&rig, vcx), [21]);
    rig.cursor_after(vcx, "{ ");
    vcx.simulate_keystrokes("y");
    vcx.run_until_parked();
    let _sent: Vec<Request> = rig.drain();
    let typed: DocVersion = rig.shown(vcx);
    assert_eq!(typed, first.next().unwrap());
    assert_eq!(
        shown_at(&rig, vcx),
        Vec::<u32>::new(),
        "old list on new text"
    );
    send(&rig, first, 21);
    vcx.run_until_parked();
    assert_eq!(
        shown_at(&rig, vcx),
        Vec::<u32>::new(),
        "a stale report shows"
    );
    send(&rig, typed, 22);
    vcx.run_until_parked();
    assert_eq!(shown_at(&rig, vcx), [22]);
}
