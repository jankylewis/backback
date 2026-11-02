use bb_text::{CharIdx, Document, LspPoint};

use super::bb_link::{shown, word_at};
use crate::intel::bb_worker::protocol::Ask;
use crate::ExplorerView;

fn doc(text: &str) -> Document {
    Document::from_text(text, None)
}

fn word(document: &Document, at: usize) -> Option<String> {
    word_at(document, CharIdx::new(at))
        .map(|(s, e): (CharIdx, CharIdx)| document.rope().slice(s.raw()..e.raw()).to_string())
}

#[test]
fn the_identifier_under_the_mouse_is_the_whole_word() {
    let text: &str = "string.IsNullOrWhiteSpace(name) ? \"there\" : my_name;";
    let d: Document = doc(text);
    assert_eq!(word(&d, 7).as_deref(), Some("IsNullOrWhiteSpace"));
    assert_eq!(word(&d, 15).as_deref(), Some("IsNullOrWhiteSpace"));
    assert_eq!(word(&d, 24).as_deref(), Some("IsNullOrWhiteSpace"));
    assert_eq!(word(&d, 0).as_deref(), Some("string"));
    assert_eq!(word(&d, 28).as_deref(), Some("name"));
    assert_eq!(word(&d, 45).as_deref(), Some("my_name"));
}

#[test]
fn punctuation_space_and_the_end_are_not_links() {
    let d: Document = doc("a.b ( ;");
    assert_eq!(word(&d, 1), None, "the dot");
    assert_eq!(word(&d, 3), None, "a space");
    assert_eq!(word(&d, 4), None, "a paren");
    assert_eq!(word(&d, 7), None, "past the end");
}

use crate::intel::bb_worker::protocol::{Answer, Request};
use crate::intel::glue::window_tests::bb_rig::{place, rig_with, Rig};
use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext};

/// Line 1 defines `Helper`; line 2 calls it. ASCII: 7.8 px per char.
const TEXT: &str = "class A {\n  void Helper() {}\n  void M() { Helper(); }\n}\n";
const CALL: usize = 13;

fn mouse_at(vcx: &mut VisualTestContext, line: usize, column: usize) -> gpui::Point<Pixels> {
    let name: &'static str = ["editor-line-0", "editor-line-1", "editor-line-2"][line];
    let row: Bounds<Pixels> = vcx.debug_bounds(name).unwrap();
    let x: f32 =
        f32::from(row.origin.x) + 50.0 + 7.8 * f32::from(u16::try_from(column).unwrap()) + 1.0;
    gpui::point(gpui::px(x), row.center().y)
}

fn hover(vcx: &mut VisualTestContext, line: usize, column: usize, mods: Modifiers) {
    let at: gpui::Point<Pixels> = mouse_at(vcx, line, column);
    vcx.simulate_mouse_move(at, None, mods);
}

fn links(rig: &mut Rig) -> Vec<LspPoint> {
    rig.drain()
        .into_iter()
        .filter_map(|r: Request| {
            if let Request::Ask {
                ask: Ask::Link(p), ..
            } = r
            {
                Some(p)
            } else {
                None
            }
        })
        .collect()
}

fn shown_text(rig: &Rig, vcx: &mut VisualTestContext) -> Option<String> {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        let doc: &Document = v.tabs.active().unwrap();
        shown(v, &rig.file)
            .map(|(s, e): (CharIdx, CharIdx)| doc.rope().slice(s.raw()..e.raw()).to_string())
    })
}

#[gpui::test]
fn cmd_hover_asks_once_per_word_and_underlines_only_a_confirmed_definition(
    cx: &mut TestAppContext,
) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
    hover(vcx, 2, CALL + 2, Modifiers::command());
    hover(vcx, 2, CALL + 4, Modifiers::command());
    let (ticket, ask): (crate::intel::bb_worker::protocol::Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::Link(p) if p == LspPoint { line: 2, character: 13 }));
    assert_eq!(
        links(&mut rig),
        Vec::<LspPoint>::new(),
        "one question per word"
    );
    assert_eq!(
        shown_text(&rig, vcx),
        None,
        "no underline before the answer"
    );
    rig.reply(vcx, ticket, Answer::Definition(vec![place(&rig.file, 1)]));
    assert_eq!(shown_text(&rig, vcx).as_deref(), Some("Helper"));
    hover(vcx, 2, CALL + 2, Modifiers::default());
    assert_eq!(shown_text(&rig, vcx), None, "Cmd released");
}

#[gpui::test]
fn a_symbol_with_no_definition_is_never_a_link(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
    hover(vcx, 0, 1, Modifiers::command());
    let (ticket, _): (crate::intel::bb_worker::protocol::Ticket, Ask) = rig.last_ask();
    rig.reply(vcx, ticket, Answer::Definition(Vec::new()));
    assert_eq!(shown_text(&rig, vcx), None);
    hover(vcx, 2, 8, Modifiers::command());
    assert_eq!(
        links(&mut rig),
        Vec::<LspPoint>::new(),
        "a space asks nothing"
    );
}

/// Readback 2026-11-03: Roslyn answers a definition (System.String) for a
/// word inside a string literal; words in strings and comments never link.
#[gpui::test]
fn words_in_strings_and_comments_never_ask_and_never_link(cx: &mut TestAppContext) {
    let text: &str = "class A {\n  string s = \"there\"; // Helper\n}\n";
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, text);
    hover(vcx, 1, 15, Modifiers::command());
    hover(vcx, 1, 26, Modifiers::command());
    assert_eq!(links(&mut rig), Vec::<LspPoint>::new());
    assert_eq!(shown_text(&rig, vcx), None);
    hover(vcx, 1, 9, Modifiers::command());
    assert_eq!(
        links(&mut rig),
        vec![LspPoint {
            line: 1,
            character: 9
        }],
        "code still asks"
    );
}

#[gpui::test]
fn cmd_click_goes_to_the_definition(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
    let at: gpui::Point<Pixels> = mouse_at(vcx, 2, CALL + 2);
    vcx.simulate_click(at, Modifiers::command());
    let (ticket, ask): (crate::intel::bb_worker::protocol::Ticket, Ask) = rig.last_ask();
    assert!(matches!(ask, Ask::Definition(p) if p.line == 2), "{ask:?}");
    rig.reply(vcx, ticket, Answer::Definition(vec![place(&rig.file, 1)]));
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(v.tabs.active().unwrap().cursor_line(), 1);
    });
}

#[gpui::test]
fn pressing_cmd_without_moving_lights_the_word_under_the_mouse(cx: &mut TestAppContext) {
    let (mut rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, TEXT);
    hover(vcx, 2, CALL + 2, Modifiers::default());
    assert_eq!(links(&mut rig), Vec::<LspPoint>::new());
    vcx.simulate_modifiers_change(Modifiers::command());
    assert_eq!(
        links(&mut rig),
        vec![LspPoint {
            line: 2,
            character: 13
        }]
    );
    vcx.simulate_modifiers_change(Modifiers::default());
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        assert_eq!(v.intel.link, None);
    });
}
