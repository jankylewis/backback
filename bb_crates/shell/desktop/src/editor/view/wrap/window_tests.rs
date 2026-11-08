//! Word wrap in a real window: a long line wraps, its end shows with no
//! sideways scroll, a click on a wrapped row lands on its chars, and off
//! brings back one row per line.

use gpui::{point, px, Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};

use super::Row;
use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

fn set_wrap(vcx: &mut VisualTestContext, on: bool) {
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| {
        let mut fonts: bb_ui::Typography = crate::settings::typography(cx);
        fonts.wrap = on;
        crate::settings::apply(cx, fonts);
    });
    vcx.run_until_parked();
}

fn rows(rig: &Rig, vcx: &mut VisualTestContext) -> Option<Vec<Row>> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let tab: &crate::editor::Tab = v.tabs.active_tab().unwrap();
        tab.wrap.as_ref().map(|w: &super::Wrapped| {
            (0..w.rows.len())
                .map(|i: usize| w.rows.get(i).unwrap())
                .collect()
        })
    })
}

fn cursor(rig: &Rig, vcx: &mut VisualTestContext) -> usize {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().cursor().raw()
    })
}

#[gpui::test]
fn a_long_line_wraps_into_rows_and_never_scrolls_sideways(cx: &mut TestAppContext) {
    let long: String = format!("class A\n{{\n    // {}\n}}\n", "word ".repeat(120));
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, &long);
    assert_eq!(rows(&rig, vcx), None, "off by default");
    set_wrap(vcx, true);
    let wrapped: Vec<Row> = rows(&rig, vcx).expect("rows while wrap is on");
    let of_line: Vec<&Row> = wrapped.iter().filter(|r: &&Row| r.line == 2).collect();
    assert!(
        of_line.len() > 2,
        "the long line takes rows: {}",
        of_line.len()
    );
    assert!(of_line[1].indent > 0.0, "later rows start at the indent");
    let line_start: usize = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().line_start(2).unwrap().raw()
    });
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
            doc.set_cursor(bb_text::CharIdx::new(line_start + 4))
                .unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
    vcx.simulate_keystrokes("cmd-right");
    vcx.run_until_parked();
    let area: Bounds<Pixels> = vcx.debug_bounds("editor-area").unwrap();
    let caret: Bounds<Pixels> = vcx.debug_bounds("editor-caret").expect("the caret");
    assert!(
        caret.left() >= area.left() && caret.right() <= area.right(),
        "the line end shows: caret {caret:?} in {area:?}"
    );
    let scrolled: f32 = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active_tab().unwrap().scroll_x
    });
    assert_eq!(scrolled, 0.0, "nothing scrolls sideways");
    let at_end: usize = cursor(&rig, vcx);
    vcx.simulate_click(caret.center(), Modifiers::default());
    assert_eq!(cursor(&rig, vcx), at_end, "a click on the caret keeps it");

    click_the_second_row(&rig, vcx, &wrapped, line_start);

    set_wrap(vcx, false);
    assert_eq!(rows(&rig, vcx), None, "off: one row per line again");
}

/// A click just right of the gutter on line 2's second row lands on that
/// row's first char.
fn click_the_second_row(
    rig: &Rig,
    vcx: &mut VisualTestContext,
    wrapped: &[Row],
    line_start: usize,
) {
    let second: usize = wrapped
        .iter()
        .position(|r: &Row| r.line == 2 && !r.first)
        .unwrap();
    let part: Row = wrapped[second];
    let row: Bounds<Pixels> = vcx
        .debug_bounds(format!("editor-line-{second}").leak())
        .expect("the second row is drawn");
    let text_x: Pixels = vcx.debug_bounds("editor-text-0").unwrap().left();
    let x: Pixels = text_x + px(part.indent + 1.0);
    vcx.simulate_click(point(x, row.center().y), Modifiers::default());
    assert_eq!(cursor(rig, vcx), line_start + part.start);
}
