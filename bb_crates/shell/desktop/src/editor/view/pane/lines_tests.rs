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

/// The selection of the tab on `name`.
fn selection_of(rig: &Rig, vcx: &mut VisualTestContext, name: &str) -> Option<(usize, usize)> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let tab: &crate::editor::Tab = v
            .tabs
            .items()
            .iter()
            .find(|tab: &&crate::editor::Tab| tab.path.ends_with(name))
            .unwrap();
        tab.document
            .selection()
            .map(|(a, b): (bb_text::CharIdx, bb_text::CharIdx)| (a.raw(), b.raw()))
    })
}

/// BB-BUG-063: dragging the line between two panes across the left pane's
/// text moved the line AND selected that text (a key then replaced it).
/// Only a press that began on the text may select by dragging.
#[gpui::test]
fn dragging_the_pane_divider_over_text_selects_nothing(cx: &mut TestAppContext) {
    let line: String = format!("    <KEY>{}</KEY>\n", "value ".repeat(60));
    let text: String = line.repeat(80);
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        crate::intel::glue::window_tests::bb_rig::rig_with(cx, false, true, &text);
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| crate::launch::bind_keys(cx));
    let path: std::path::PathBuf = rig.root.join("B.cs");
    std::fs::write(&path, "class B {}\n").unwrap();
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            v.tabs.open(&path).unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
    vcx.simulate_keystrokes("ctrl-cmd-right");
    // As in the report: the left pane has the focus, its cursor on a line.
    let left: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("pane-0").unwrap();
    let inside: gpui::Point<gpui::Pixels> = left.center();
    vcx.simulate_click(inside, Modifiers::default());
    assert_eq!(
        selection_of(&rig, vcx, "A.cs"),
        None,
        "a click selects nothing"
    );
    for by in [-120.0, 80.0] {
        drag(
            vcx,
            "pane-divider-0",
            gpui::point(gpui::px(by), gpui::px(0.0)),
        );
        assert_eq!(selection_of(&rig, vcx, "A.cs"), None, "drag by {by}");
    }
    let start: gpui::Point<gpui::Pixels> = vcx.debug_bounds("editor-text-2").unwrap().origin;
    let from: gpui::Point<gpui::Pixels> = start + gpui::point(gpui::px(4.0), gpui::px(4.0));
    vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    let to: gpui::Point<gpui::Pixels> = from + gpui::point(gpui::px(60.0), gpui::px(0.0));
    vcx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::default());
    vcx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
    let shown: bool =
        selection_of(&rig, vcx, "A.cs").is_some() || selection_of(&rig, vcx, "B.cs").is_some();
    assert!(shown, "a drag that starts on the text still selects");
}
