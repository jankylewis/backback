//! BB-BUG-040 through the window: a tab dragged onto the right half splits
//! the editor; a press in the other pane focuses it and places the cursor
//! there; the move keys move the active tab between the groups.

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

/// `A.cs` (the rig's) plus `B.cs` with three lines, `B.cs` active.
fn two_tabs(cx: &mut TestAppContext) -> (Rig, &mut VisualTestContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    let path: std::path::PathBuf = rig.root.join("B.cs");
    std::fs::write(&path, "one\ntwo\nthree\n").unwrap();
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            v.tabs.open(&path).unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
    (rig, vcx)
}

/// (the focused group's active file, whether split).
fn seen(rig: &Rig, vcx: &mut VisualTestContext) -> (String, bool) {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let name: String = v
            .tabs
            .active_tab()
            .map_or_else(String::new, |tab: &crate::editor::Tab| {
                tab.path.file_name().unwrap().to_string_lossy().into_owned()
            });
        (name, v.tabs.is_split())
    })
}

fn centre(vcx: &mut VisualTestContext, selector: &'static str) -> gpui::Point<gpui::Pixels> {
    vcx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("no {selector}"))
        .center()
}

#[gpui::test]
fn a_tab_dragged_onto_the_right_half_opens_a_right_pane(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = two_tabs(cx);
    assert!(vcx.debug_bounds("pane-right").is_none());
    let from: gpui::Point<gpui::Pixels> = centre(vcx, "tab-1");
    let drag: Option<MouseButton> = Some(MouseButton::Left);
    vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_move(
        from + gpui::point(gpui::px(20.0), gpui::px(40.0)),
        drag,
        Modifiers::default(),
    );
    vcx.run_until_parked();
    let to: gpui::Point<gpui::Pixels> = centre(vcx, "drop-right");
    vcx.simulate_mouse_move(to, drag, Modifiers::default());
    vcx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
    assert_eq!(
        seen(&rig, vcx),
        ("B.cs".to_owned(), true),
        "B moved right, focused"
    );
    let left: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("pane-left").unwrap();
    let right: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("pane-right").unwrap();
    assert!(right.origin.x > left.origin.x, "two panes side by side");
    assert!(
        (left.size.width - right.size.width).abs() < gpui::px(2.0),
        "half and half"
    );
}

#[gpui::test]
fn a_press_in_the_other_pane_focuses_it_and_places_the_cursor(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = two_tabs(cx);
    vcx.simulate_keystrokes("ctrl-cmd-right");
    assert_eq!(seen(&rig, vcx), ("B.cs".to_owned(), true));
    // Focus back left with a press on its pane, then press line 3 of B.
    let left: gpui::Point<gpui::Pixels> = centre(vcx, "pane-body-left");
    vcx.simulate_click(left, Modifiers::default());
    assert_eq!(seen(&rig, vcx).0, "A.cs", "the left pane took focus");
    let row: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("pane-body-right").unwrap();
    let line_height: gpui::Pixels = vcx.debug_bounds("editor-line-0").unwrap().size.height;
    let at: gpui::Point<gpui::Pixels> = gpui::point(
        row.origin.x + gpui::px(120.0),
        row.origin.y + line_height * 2.5,
    );
    vcx.simulate_click(at, Modifiers::default());
    let (line, name): (usize, String) =
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
            (
                tab.document.cursor_line(),
                tab.path.file_name().unwrap().to_string_lossy().into_owned(),
            )
        });
    assert_eq!(
        (name.as_str(), line),
        ("B.cs", 2),
        "one press: focus and cursor on line 3"
    );
    vcx.simulate_keystrokes("x");
    let (a, b): (String, String) = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let texts: Vec<String> = v
            .tabs
            .items()
            .iter()
            .map(|tab: &crate::editor::Tab| tab.document.text())
            .collect();
        (
            texts.first().unwrap().clone(),
            texts.get(1).unwrap().clone(),
        )
    });
    assert_eq!(a, "class A {}\n", "typing never reaches the other pane");
    assert!(b.contains('x'), "{b:?}");
}

#[gpui::test]
fn cmd_backslash_moves_the_tab_back_and_the_split_ends(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = two_tabs(cx);
    vcx.simulate_keystrokes("cmd-\\");
    assert_eq!(seen(&rig, vcx), ("B.cs".to_owned(), true));
    vcx.simulate_keystrokes("cmd-\\");
    assert_eq!(
        seen(&rig, vcx),
        ("B.cs".to_owned(), false),
        "back left: one pane"
    );
    assert!(vcx.debug_bounds("pane-right").is_none());
}
