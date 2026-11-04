//! BB-BUG-039 through the window: a right-click on a tab opens its menu, a
//! row click acts, and Cmd+K Shift+Enter pins without editing the text.

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

fn names(rig: &Rig, vcx: &mut VisualTestContext) -> Vec<String> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs
            .items()
            .iter()
            .map(|tab: &crate::editor::Tab| {
                let name: String = tab.path.file_name().unwrap().to_string_lossy().into_owned();
                if tab.pinned {
                    format!("{name}*")
                } else {
                    name
                }
            })
            .collect()
    })
}

/// A window on `A.cs` plus `B.cs` and `C.cs`, `C.cs` active.
fn three_tabs(cx: &mut TestAppContext) -> (Rig, &mut VisualTestContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    for name in ["B.cs", "C.cs"] {
        let path: std::path::PathBuf = rig.root.join(name);
        std::fs::write(&path, "class X {}\n").unwrap();
        rig.view.update(
            vcx,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                v.tabs.open(&path).unwrap();
                cx.notify();
            },
        );
    }
    vcx.run_until_parked();
    (rig, vcx)
}

fn press(vcx: &mut VisualTestContext, selector: &'static str, button: MouseButton) {
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_mouse_down(at, button, Modifiers::default());
    vcx.simulate_mouse_up(at, button, Modifiers::default());
    vcx.run_until_parked();
}

#[gpui::test]
fn a_right_click_opens_the_tab_menu_and_close_others_keeps_that_tab(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = three_tabs(cx);
    assert_eq!(names(&rig, vcx), ["A.cs", "B.cs", "C.cs"]);
    assert!(vcx.debug_bounds("tab-menu").is_none(), "no menu before");
    press(vcx, "tab-1", MouseButton::Right);
    assert!(
        vcx.debug_bounds("tab-menu").is_some(),
        "the right-click opened the menu"
    );
    press(vcx, "tab-menu-pin", MouseButton::Left);
    assert_eq!(names(&rig, vcx), ["B.cs*", "A.cs", "C.cs"]);
    assert!(
        vcx.debug_bounds("tab-menu").is_none(),
        "a row click closes the menu"
    );
    press(vcx, "tab-2", MouseButton::Right);
    press(vcx, "tab-menu-close-others", MouseButton::Left);
    assert_eq!(names(&rig, vcx), ["B.cs*", "C.cs"], "the pinned tab stays");
    press(vcx, "close-0", MouseButton::Left);
    assert_eq!(
        names(&rig, vcx),
        ["B.cs", "C.cs"],
        "the pin unpins, not closes"
    );
    press(vcx, "tab-0", MouseButton::Right);
    press(vcx, "tab-menu-close-all", MouseButton::Left);
    assert_eq!(names(&rig, vcx), Vec::<String>::new());
}

#[gpui::test]
fn cmd_k_shift_enter_pins_the_active_tab_and_leaves_its_text(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = three_tabs(cx);
    let before: String = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().text()
    });
    vcx.simulate_keystrokes("cmd-k shift-enter");
    assert_eq!(names(&rig, vcx), ["C.cs*", "A.cs", "B.cs"]);
    let after: String = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().text()
    });
    assert_eq!(after, before, "Shift+Enter of the chord inserted no line");
    vcx.simulate_keystrokes("cmd-alt-t");
    assert_eq!(names(&rig, vcx), ["C.cs*"], "Close Others keeps pins");
    vcx.simulate_keystrokes("cmd-k cmd-w");
    assert_eq!(
        names(&rig, vcx),
        Vec::<String>::new(),
        "Close All takes pins too"
    );
}
