//! BB-BUG-044 and BB-BUG-045 through the window: the lines between the
//! panes and the tool window's edge drag; three panes share the editor.

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use super::window_tests::two_tabs;
use crate::intel::glue::window_tests::bb_rig::Rig;
use crate::ExplorerView;

pub(crate) fn drag(
    vcx: &mut VisualTestContext,
    selector: &'static str,
    by: gpui::Point<gpui::Pixels>,
) {
    let from: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    let button: Option<MouseButton> = Some(MouseButton::Left);
    vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_move(from + by, button, Modifiers::default());
    vcx.simulate_mouse_up(from + by, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
}

fn width(vcx: &mut VisualTestContext, selector: &'static str) -> f32 {
    f32::from(vcx.debug_bounds(selector).unwrap().size.width)
}

/// A window with `A.cs` and `B.cs`, `B.cs` in a right pane.
pub(crate) fn split_ab(cx: &mut TestAppContext) -> (Rig, &mut VisualTestContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = two_tabs(cx);
    vcx.simulate_keystrokes("ctrl-cmd-right");
    (rig, vcx)
}

#[gpui::test]
fn the_line_between_the_panes_and_the_tool_window_edge_drag(cx: &mut TestAppContext) {
    let (_rig, vcx): (Rig, &mut VisualTestContext) = split_ab(cx);
    let (left, right): (f32, f32) = (width(vcx, "pane-0"), width(vcx, "pane-1"));
    drag(
        vcx,
        "pane-divider-0",
        gpui::point(gpui::px(60.0), gpui::px(0.0)),
    );
    let (left2, right2): (f32, f32) = (width(vcx, "pane-0"), width(vcx, "pane-1"));
    assert!((left2 - left - 60.0).abs() <= 1.0, "left {left} -> {left2}");
    assert!(
        (right - right2 - 60.0).abs() <= 1.0,
        "right {right} -> {right2}"
    );
    drag(
        vcx,
        "explorer-resize-edge",
        gpui::point(gpui::px(-40.0), gpui::px(0.0)),
    );
    assert!(
        width(vcx, "pane-0") > left2,
        "a narrower tool window widens the panes"
    );
}

/// BB-BUG-045: three panes side by side, each line between them drags,
/// and a pane left empty closes (the panes right of it move left).
#[gpui::test]
fn three_panes_share_the_editor_and_their_lines_drag(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = two_tabs(cx);
    let path: std::path::PathBuf = rig.root.join("C.cs");
    std::fs::write(&path, "class C {}\n").unwrap();
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            v.tabs.open(&path).unwrap();
            cx.notify();
        },
    );
    vcx.simulate_keystrokes("cmd-\\");
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            v.tabs.switch(1);
            cx.notify();
        },
    );
    vcx.simulate_keystrokes("ctrl-cmd-right cmd-\\");
    let widths: Vec<f32> = ["pane-0", "pane-1", "pane-2"]
        .iter()
        .map(|p: &&str| width(vcx, p))
        .collect();
    let (least, most): (f32, f32) = (
        widths.iter().copied().fold(f32::MAX, f32::min),
        widths.iter().copied().fold(0.0, f32::max),
    );
    assert!(most - least <= 2.0, "thirds: {widths:?}");
    drag(
        vcx,
        "pane-divider-1",
        gpui::point(gpui::px(-40.0), gpui::px(0.0)),
    );
    assert!(
        (width(vcx, "pane-0") - widths[0]).abs() <= 1.0,
        "the first pane stays"
    );
    assert!((widths[1] - width(vcx, "pane-1") - 40.0).abs() <= 1.0);
    assert!((width(vcx, "pane-2") - widths[2] - 40.0).abs() <= 1.0);
    vcx.simulate_keystrokes("cmd-\\");
    assert!(
        vcx.debug_bounds("pane-2").is_none(),
        "from the third pane Cmd+\\ moves left"
    );
}

/// BB-BUG-054: selected text, of any kind, is the theme's plain text on its
/// `selection` colour (it was the syntax colour on the light accent).
#[test]
fn selected_text_is_plain_text_on_the_selection_colour() {
    for theme in [bb_ui::bb_theme::DARK, bb_ui::bb_theme::LIGHT] {
        for (kind, link) in [
            (bb_syntax::HighlightKind::Keyword, false),
            (bb_syntax::HighlightKind::Comment, false),
            (bb_syntax::HighlightKind::Plain, true),
        ] {
            let run: super::super::bb_line::Run = super::super::bb_line::Run {
                text: "AIQ_ENV".to_owned(),
                kind,
                selected: true,
                mark: None,
                link,
            };
            let styled: gpui::TextRun = super::super::bb_line::text_run(&run, theme, "Menlo");
            assert_eq!(styled.color, crate::color(theme.text).into(), "{kind:?}");
            assert_eq!(
                styled.background_color,
                Some(crate::color(theme.selection).into())
            );
        }
    }
}
