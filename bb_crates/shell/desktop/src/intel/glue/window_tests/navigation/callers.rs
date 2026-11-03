//! Show Call Hierarchy (Shift+Alt+H), BBEOD24: the callers in the places
//! panel, even one; none says so in the status bar.

use std::path::PathBuf;

use super::super::bb_rig::{place, rig, Rig};
use crate::intel::bb_worker::protocol::{Answer, Ask, Place, Ticket};
use crate::intel::model::popups::PlacesPanel;
use crate::ExplorerView;
use gpui::{TestAppContext, VisualTestContext};

/// Shift+Alt+H as macOS reports it on a US layout: the key `h` and the
/// char Option+Shift+H types, `Ó` (a key the editor would insert).
fn shift_alt_h() -> gpui::Keystroke {
    gpui::Keystroke {
        modifiers: gpui::Modifiers {
            alt: true,
            shift: true,
            ..gpui::Modifiers::default()
        },
        key: String::from("h"),
        key_char: Some(String::from("Ó")),
    }
}

fn callers(count: usize, places: Vec<Place>) -> Answer {
    Answer::Callers {
        name: String::from("Greeter.Hello(string)"),
        callers: count,
        places,
    }
}

#[gpui::test]
fn shift_alt_h_passes_the_app_keys_and_lists_one_caller_without_a_jump(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    // The rig installs no app bindings (BB-UIX-010): install the app's own.
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    let before: String = rig.text(vcx);
    vcx.update(|window: &mut gpui::Window, cx: &mut gpui::App| {
        window.dispatch_keystroke(shift_alt_h(), cx);
    });
    vcx.run_until_parked();
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::Callers(_)), "{ask:?}");
    assert_eq!(rig.text(vcx), before, "Shift+Alt+H typed no text");
    let spec: PathBuf = rig.root.join("Spec.cs");
    std::fs::write(&spec, "class Spec\n{\n    void Greets() { }\n}\n").unwrap();
    rig.reply(vcx, ticket, callers(1, vec![place(&spec, 2)]));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let panel: &PlacesPanel = v.intel.references.as_ref().unwrap();
        assert_eq!(panel.title, "1 caller of Greeter.Hello(string)");
        assert_eq!(panel.places.len(), 1);
        assert_eq!(v.tabs.active_tab().unwrap().path, rig.file, "no jump");
    });
}

#[gpui::test]
fn no_callers_says_so_and_escape_closes_a_list_of_two(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.simulate_keystrokes("shift-alt-h");
    let (ticket, _): (Ticket, Ask) = rig.last_ask();
    rig.reply(vcx, ticket, callers(0, Vec::new()));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(v.status.as_deref(), Some("No callers found"));
        assert!(v.intel.references.is_none());
    });
    vcx.simulate_keystrokes("shift-alt-h");
    let (ticket, _): (Ticket, Ask) = rig.last_ask();
    let file: PathBuf = rig.file.clone();
    rig.reply(
        vcx,
        ticket,
        callers(2, vec![place(&file, 0), place(&file, 0)]),
    );
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let panel: &PlacesPanel = v.intel.references.as_ref().unwrap();
        assert_eq!(panel.title, "2 callers of Greeter.Hello(string)");
    });
    vcx.simulate_keystrokes("escape");
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert!(v.intel.references.is_none());
    });
}
