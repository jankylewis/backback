//! Go to Implementation (Cmd+Alt+B) and Go to Type Definition (Cmd+Ctrl+B),
//! BBEOD22.

use std::path::PathBuf;

use super::super::bb_rig::{place, rig, Rig};
use crate::intel::bb_worker::protocol::{Answer, Ask, Ticket};
use crate::ExplorerView;
use gpui::{TestAppContext, VisualTestContext};

/// A four-line file next to `A.cs` to jump into.
fn target(rig: &Rig) -> PathBuf {
    let path: PathBuf = rig.root.join("B.cs");
    std::fs::write(&path, "class B\n{\n    void M() { }\n}\n").unwrap();
    path
}

#[gpui::test]
fn cmd_alt_b_jumps_to_one_implementation_and_lists_several(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    let b: PathBuf = target(&rig);
    vcx.simulate_keystrokes("cmd-alt-b");
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::Implementation(_)), "{ask:?}");
    rig.reply(vcx, ticket, Answer::Definition(vec![place(&b, 2)]));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
        assert_eq!(tab.path, b);
        assert_eq!(tab.document.cursor_line(), 2);
        assert!(v.intel.references.is_none());
    });
    vcx.simulate_keystrokes("cmd-alt-b");
    let (ticket, _): (Ticket, Ask) = rig.last_ask();
    rig.reply(
        vcx,
        ticket,
        Answer::Definition(vec![place(&b, 0), place(&b, 2)]),
    );
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let panel: &crate::intel::model::popups::PlacesPanel = v.intel.references.as_ref().unwrap();
        assert_eq!(panel.title, "2 implementations");
        assert_eq!(panel.places.len(), 2);
    });
}

#[gpui::test]
fn cmd_ctrl_b_asks_for_the_type_definition_and_ctrl_b_still_moves_left(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.simulate_keystrokes("cmd-ctrl-b");
    let (ticket, ask): (Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::TypeDefinition(_)), "{ask:?}");
    rig.reply(vcx, ticket, Answer::Definition(Vec::new()));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(v.status.as_deref(), Some("No type definition found"));
    });
    vcx.simulate_keystrokes("end ctrl-b");
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
        let end: usize = tab.document.text().trim_end().chars().count();
        assert_eq!(
            tab.document.cursor().raw(),
            end - 1,
            "Ctrl+B moved one char left"
        );
    });
}
