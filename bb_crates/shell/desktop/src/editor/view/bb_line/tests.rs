//! BB-BUG-005 / BB-UIX-001: the cursor never moves the text. The test text
//! system gives 7.8 px per BMP char and 15.6 px per astral char (emoji), so
//! a constant char width cannot pass these tests by luck.

use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::intel::view::marks::{Ink, Mark};
use crate::ExplorerView;

/// A tab, an emoji, a word: the shapes that break column math.
const LINE: &str = "\tvar s = \"😀\"; CapturedOutboundRequest(";
const TEXT: &str = "\tvar s = \"😀\"; CapturedOutboundRequest(\n\nx\n";

/// The x of char boundary `column` on `LINE`, from the test font alone.
fn shaped_x(column: usize) -> f32 {
    LINE.chars()
        .take(column)
        .map(|ch: char| 7.8 * f32::from(u8::try_from(ch.len_utf16()).unwrap()))
        .sum()
}

fn open(cx: &mut TestAppContext) -> (Rig, &mut VisualTestContext) {
    rig_with(cx, false, true, TEXT)
}

fn put_cursor(rig: &Rig, vcx: &mut VisualTestContext, line: usize, column: usize) {
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
            let start: usize = doc.line_start(line).unwrap().raw();
            doc.set_cursor(bb_text::CharIdx::new(start + column))
                .unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
}

fn line_text(rig: &Rig, vcx: &mut VisualTestContext, marks: &[Mark]) -> String {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        super::styled_line(v.tabs.active_tab().unwrap(), 0, marks)
            .into_iter()
            .map(|run: super::Run| run.text)
            .collect()
    })
}

#[gpui::test]
fn the_line_text_is_the_file_line_at_every_cursor_column(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = open(cx);
    let squiggle: Vec<Mark> = vec![Mark {
        from: 1,
        to: 12,
        ink: Ink::Error,
    }];
    for column in 0..=LINE.chars().count() {
        put_cursor(&rig, vcx, 0, column);
        assert_eq!(line_text(&rig, vcx, &[]), LINE, "cursor at column {column}");
        assert_eq!(
            line_text(&rig, vcx, &squiggle),
            LINE,
            "marks, column {column}"
        );
    }
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, _: &mut gpui::Context<ExplorerView>| {
            let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
            doc.select(bb_text::CharIdx::new(2), bb_text::CharIdx::new(14))
                .unwrap();
        },
    );
    assert_eq!(line_text(&rig, vcx, &[]), LINE, "with a selection");
}

#[gpui::test]
fn the_caret_sits_at_the_shaped_x_of_its_column_and_the_text_never_moves(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = open(cx);
    let mut text_boxes: Vec<Bounds<Pixels>> = Vec::new();
    for column in 0..=LINE.chars().count() {
        put_cursor(&rig, vcx, 0, column);
        let row: Bounds<Pixels> = vcx.debug_bounds("editor-line-0").expect("line row");
        let caret: Bounds<Pixels> = vcx.debug_bounds("editor-caret").expect("caret quad");
        let want: f32 = f32::from(row.origin.x) + 50.0 + shaped_x(column);
        // Layout snaps an element to whole device pixels (scale 1 here); glyphs
        // keep fractional x. So the caret is within half a device pixel.
        assert!(
            (f32::from(caret.origin.x) - want).abs() <= 0.5,
            "column {column}: {caret:?} want {want}"
        );
        assert!(
            f32::from(caret.size.width) <= 2.0,
            "a thin quad, not a glyph"
        );
        text_boxes.push(vcx.debug_bounds("editor-text-0").expect("line text"));
    }
    let first: Bounds<Pixels> = text_boxes[0];
    // A text element's width is the shaped width rounded up to a whole pixel;
    // a caret glyph in the text would add a full 7.8 px char.
    let width: f32 = f32::from(first.size.width) - shaped_x(LINE.chars().count());
    assert!((0.0..1.0).contains(&width), "{first:?}");
    assert!(
        text_boxes.iter().all(|b: &Bounds<Pixels>| *b == first),
        "{text_boxes:?}"
    );
    put_cursor(&rig, vcx, 1, 0);
    let row: Bounds<Pixels> = vcx.debug_bounds("editor-line-1").expect("empty row");
    let caret: Bounds<Pixels> = vcx
        .debug_bounds("editor-caret")
        .expect("caret on empty line");
    assert!((f32::from(caret.origin.x) - (f32::from(row.origin.x) + 50.0)).abs() <= 0.5);
}

#[gpui::test]
fn a_click_lands_on_the_column_drawn_under_the_mouse(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = open(cx);
    put_cursor(&rig, vcx, 2, 0);
    let row: Bounds<Pixels> = vcx.debug_bounds("editor-line-0").expect("line row");
    for column in [0, 10, 11, 14, LINE.chars().count() - 1] {
        let x: f32 = f32::from(row.origin.x) + 50.0 + shaped_x(column) + 1.0;
        let at: gpui::Point<Pixels> = gpui::point(gpui::px(x), row.center().y);
        vcx.simulate_click(at, Modifiers::default());
        let got: usize = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            v.tabs.active().unwrap().cursor().raw()
        });
        assert_eq!(got, column, "click at x={x}");
    }
}

#[gpui::test]
fn a_cursor_popup_is_anchored_at_the_caret_x(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = open(cx);
    let column: usize = 14;
    put_cursor(&rig, vcx, 0, column);
    vcx.simulate_keystrokes("ctrl-space");
    let (ticket, _): (_, _) = rig.last_ask();
    rig.reply(
        vcx,
        ticket,
        crate::intel::glue::window_tests::bb_rig::items(&["Run"]),
    );
    let caret: Bounds<Pixels> = vcx.debug_bounds("editor-caret").expect("caret quad");
    let popup: Bounds<Pixels> = vcx.debug_bounds("cursor-popup").expect("popup anchor");
    assert!(
        (f32::from(popup.origin.x) - f32::from(caret.origin.x)).abs() < 0.01,
        "{popup:?} vs {caret:?}"
    );
}

fn cursor(rig: &Rig, vcx: &mut VisualTestContext) -> usize {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().cursor().raw()
    })
}

/// BB-BUG-007: Home, End and the Cmd+arrows reach the cursor through the
/// editor's own key handling, as typed.
#[gpui::test]
fn home_end_and_cmd_arrows_move_the_cursor(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = open(cx);
    let end: usize = LINE.chars().count();
    put_cursor(&rig, vcx, 0, 14);
    vcx.simulate_keystrokes("end");
    assert_eq!(cursor(&rig, vcx), end);
    vcx.simulate_keystrokes("home");
    assert_eq!(cursor(&rig, vcx), 1, "first non-blank after the tab");
    vcx.simulate_keystrokes("cmd-right");
    assert_eq!(cursor(&rig, vcx), end);
    vcx.simulate_keystrokes("cmd-left cmd-left");
    assert_eq!(cursor(&rig, vcx), 0, "smart Home twice: column 0");
    vcx.simulate_keystrokes("cmd-down");
    assert_eq!(cursor(&rig, vcx), TEXT.chars().count());
    vcx.simulate_keystrokes("cmd-up");
    assert_eq!(cursor(&rig, vcx), 0);
}

/// BBEOD14: zoom changes the one size every surface reads: the row height,
/// the gutter, the caret and the text all scale; nothing stays at 13 pt.
#[gpui::test]
fn zoom_scales_the_row_the_gutter_the_caret_and_the_text(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = open(cx);
    put_cursor(&rig, vcx, 0, 0);
    let row_13: Bounds<Pixels> = vcx.debug_bounds("editor-line-0").expect("row");
    let text_13: Bounds<Pixels> = vcx.debug_bounds("editor-text-0").expect("text");
    let zoomed: bb_ui::Typography = bb_ui::Typography {
        zoom: 13,
        ..bb_ui::Typography::default()
    };
    vcx.update(|_: &mut gpui::Window, cx: &mut gpui::App| {
        crate::settings::apply(cx, zoomed.clone());
    });
    vcx.run_until_parked();
    let row_26: Bounds<Pixels> = vcx.debug_bounds("editor-line-0").expect("row");
    let text_26: Bounds<Pixels> = vcx.debug_bounds("editor-text-0").expect("text");
    let caret_26: Bounds<Pixels> = vcx.debug_bounds("editor-caret").expect("caret");
    assert_eq!(f32::from(row_13.size.height), 22.0);
    assert_eq!(f32::from(row_26.size.height), zoomed.line_height());
    let ratio: f32 = f32::from(text_26.size.width) / f32::from(text_13.size.width);
    assert!((ratio - 2.0).abs() < 0.05, "text width x{ratio}, want x2");
    let gutter: f32 = f32::from(caret_26.origin.x) - f32::from(row_26.origin.x);
    assert!(
        (gutter - zoomed.gutter_width()).abs() <= 0.5,
        "caret after a {gutter} px gutter"
    );
    assert_eq!(f32::from(caret_26.size.height), zoomed.size() + 3.0);
}
