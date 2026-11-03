//! BB-BUG-011: Option+arrows move by word and Option+Backspace removes one,
//! through the editor's own key handling (they moved one char before).

use gpui::{TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

const TEXT: &str = "\tvar s = 1;\n";

fn text(rig: &Rig, vcx: &mut VisualTestContext) -> String {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().text()
    })
}

fn put(rig: &Rig, vcx: &mut VisualTestContext, anchor: usize, cursor: usize) {
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let doc: &mut bb_text::Document = v.tabs.active_mut().unwrap();
            doc.select(bb_text::CharIdx::new(anchor), bb_text::CharIdx::new(cursor))
                .unwrap();
            cx.notify();
        },
    );
    vcx.run_until_parked();
}

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

/// The macOS text-system Ctrl keys (Cocoa, VS Code on macOS): Ctrl+A/E line
/// start/end, Ctrl+B/F char left/right, Ctrl+P/N line up/down, Ctrl+H/D
/// delete back/forward. Before, the editor ignored them.
#[gpui::test]
fn ctrl_emacs_keys_move_and_delete(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "\tab\ncd\n");
    vcx.simulate_keystrokes("ctrl-e");
    assert_eq!(cursor(&rig, vcx), 3, "line end");
    vcx.simulate_keystrokes("ctrl-a");
    assert_eq!(
        cursor(&rig, vcx),
        0,
        "line start, column 0 (not smart Home)"
    );
    vcx.simulate_keystrokes("ctrl-f ctrl-f");
    assert_eq!(cursor(&rig, vcx), 2);
    vcx.simulate_keystrokes("ctrl-b");
    assert_eq!(cursor(&rig, vcx), 1);
    vcx.simulate_keystrokes("ctrl-n");
    assert_eq!(cursor(&rig, vcx), 5, "next line, same column");
    vcx.simulate_keystrokes("ctrl-p");
    assert_eq!(cursor(&rig, vcx), 1);
    vcx.simulate_keystrokes("ctrl-d ctrl-h");
    let text: String = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.tabs.active().unwrap().text()
    });
    assert_eq!(text, "b\ncd\n", "Ctrl+D took `a`, Ctrl+H the tab");
}

/// BB-BUG-015: Tab typed a tab over the selection; now it indents the
/// selected lines and Shift+Tab outdents them, the text kept.
#[gpui::test]
fn tab_with_a_selection_indents_and_shift_tab_outdents(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "class C\n{\n    int a;\n}\n");
    put(&rig, vcx, 10, 13);
    vcx.simulate_keystrokes("tab");
    assert_eq!(
        text(&rig, vcx),
        "class C\n{\n        int a;\n}\n",
        "indented, nothing lost"
    );
    vcx.simulate_keystrokes("shift-tab");
    assert_eq!(text(&rig, vcx), "class C\n{\n    int a;\n}\n", "outdented");
    vcx.simulate_keystrokes("shift-tab");
    assert_eq!(text(&rig, vcx), "class C\n{\nint a;\n}\n", "and again");
}

/// BB-BUG-014: Cmd+Backspace did nothing; Cmd+Delete is its mirror.
#[gpui::test]
fn cmd_backspace_and_cmd_delete_reach_the_line_edges(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "    int alpha = beta;\n");
    put(&rig, vcx, 16, 16);
    vcx.simulate_keystrokes("cmd-backspace");
    assert_eq!(text(&rig, vcx), "beta;\n", "to the line start");
    vcx.simulate_keystrokes("cmd-delete");
    assert_eq!(text(&rig, vcx), "\n", "to the line end");
}

/// BB-BUG-016 and BB-BUG-017: Cmd+A selects all; Cmd+/ toggles a line
/// comment; neither did anything.
#[gpui::test]
fn cmd_a_selects_all_and_cmd_slash_toggles_a_line_comment(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "class C\n{\n    int a;\n}\n");
    put(&rig, vcx, 14, 14);
    vcx.simulate_keystrokes("cmd-/");
    assert_eq!(
        text(&rig, vcx),
        "class C\n{\n    // int a;\n}\n",
        "commented"
    );
    vcx.simulate_keystrokes("cmd-/");
    assert_eq!(text(&rig, vcx), "class C\n{\n    int a;\n}\n", "and back");
    vcx.simulate_keystrokes("cmd-a");
    vcx.simulate_keystrokes("x");
    assert_eq!(
        text(&rig, vcx),
        "x",
        "select all, then the typed char replaces it"
    );
}

#[gpui::test]
fn the_line_command_keys_reach_the_line_commands(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "class C\n{\n    int a;\n}\n");
    put(&rig, vcx, 14, 14);
    vcx.simulate_keystrokes("cmd-enter q");
    assert_eq!(
        text(&rig, vcx),
        "class C\n{\n    int a;\n    q\n}\n",
        "Cmd+Enter"
    );
    vcx.simulate_keystrokes("cmd-shift-enter w");
    assert_eq!(
        text(&rig, vcx),
        "class C\n{\n    int a;\n    w\n    q\n}\n",
        "Cmd+Shift+Enter"
    );
    vcx.simulate_keystrokes("cmd-shift-k cmd-shift-k");
    assert_eq!(
        text(&rig, vcx),
        "class C\n{\n    int a;\n}\n",
        "Cmd+Shift+K twice"
    );
}

#[gpui::test]
fn cmd_l_selects_the_line_and_a_key_replaces_it(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "class C\n{\n    int a;\n}\n");
    put(&rig, vcx, 14, 14);
    vcx.simulate_keystrokes("cmd-l x");
    assert_eq!(
        text(&rig, vcx),
        "class C\n{\nx}\n",
        "the whole line, break included"
    );
}

/// BB-BUG-022: a click right of a line's last char, inside the editor, puts
/// the cursor at the line's end; on an empty line, at its start (VS Code).
#[gpui::test]
fn a_click_past_the_line_end_puts_the_cursor_at_the_end(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) =
        rig_with(cx, false, true, "class C\n{\n    int a;\n\n}\n");
    let area: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("editor-area").unwrap();
    let past: gpui::Pixels = area.right() - gpui::px(12.0);
    for (selector, expected) in [("editor-line-2", 20_usize), ("editor-line-3", 21)] {
        let row: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds(selector).unwrap();
        vcx.simulate_click(
            gpui::point(past, row.center().y),
            gpui::Modifiers::default(),
        );
        assert_eq!(
            cursor(&rig, vcx),
            expected,
            "a click past the end of {selector}"
        );
    }
}
