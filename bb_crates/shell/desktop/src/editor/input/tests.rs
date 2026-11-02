//! BB-BUG-011: Option+arrows move by word and Option+Backspace removes one,
//! through the editor's own key handling (they moved one char before).

use gpui::{TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

const TEXT: &str = "\tvar s = 1;\n";

fn cursor(rig: &Rig, vcx: &mut VisualTestContext) -> usize {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().cursor().raw()
    })
}

#[gpui::test]
fn option_arrows_and_option_backspace_work_by_word(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
            doc.set_cursor(bb_text::CharIdx::new(1)).unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
    vcx.simulate_keystrokes("alt-right");
    assert_eq!(cursor(&rig, vcx), 4, "after `var`");
    vcx.simulate_keystrokes("alt-right");
    assert_eq!(cursor(&rig, vcx), 6, "after `s`");
    vcx.simulate_keystrokes("alt-left");
    assert_eq!(cursor(&rig, vcx), 5, "before `s`");
    vcx.simulate_keystrokes("alt-backspace");
    let text: String = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().text()
    });
    assert_eq!(text, "\ts = 1;\n", "`var ` removed");
}
