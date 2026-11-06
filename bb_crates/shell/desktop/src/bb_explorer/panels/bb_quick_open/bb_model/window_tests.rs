//! BB-BUG-057: the Find in Files box takes a paste and has a caret that
//! the arrow keys move, in the real window.

use crate::intel::glue::window_tests::bb_rig::{rig, Rig};

fn query(rig: &Rig, vcx: &mut gpui::VisualTestContext) -> (String, usize) {
    rig.view
        .read_with(vcx, |v: &crate::ExplorerView, _: &gpui::App| {
            let quick: &super::QuickOpen = v.quick_open.as_ref().unwrap();
            (quick.query.clone(), quick.caret)
        })
}

fn caret_x(vcx: &mut gpui::VisualTestContext) -> f32 {
    vcx.run_until_parked();
    f32::from(vcx.debug_bounds("quick-open-caret").unwrap().origin.x)
}

#[gpui::test]
fn the_find_box_pastes_at_a_caret_the_arrows_move(cx: &mut gpui::TestAppContext) {
    let (rig, vcx): (Rig, &mut gpui::VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(crate::quick_open::FindInFiles);
    vcx.simulate_keystrokes("a b");
    vcx.write_to_clipboard(gpui::ClipboardItem::new_string("Needle\n".to_owned()));
    vcx.simulate_keystrokes("left cmd-v");
    assert_eq!(query(&rig, vcx), ("aNeedleb".to_owned(), 7));
    let end: f32 = caret_x(vcx);
    vcx.simulate_keystrokes("alt-left");
    assert_eq!(query(&rig, vcx).1, 0, "aNeedle is one word");
    let start: f32 = caret_x(vcx);
    assert!(
        start < end,
        "the caret is drawn where it is: {start} < {end}"
    );
    vcx.simulate_keystrokes("right delete x end backspace");
    assert_eq!(query(&rig, vcx), ("axeedle".to_owned(), 7));
    vcx.simulate_keystrokes("cmd-left y cmd-right z");
    assert_eq!(query(&rig, vcx), ("yaxeedlez".to_owned(), 9));
}
