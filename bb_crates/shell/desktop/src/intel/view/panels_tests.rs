//! BB-BUG-044 through the window: the panel under the editor drags taller
//! by its top edge, lists the match first and scrolls sideways to the rest
//! of a long row.

use gpui::{TestAppContext, VisualTestContext};

use crate::editor::pane_tests::{drag, split_ab};
use crate::intel::glue::window_tests::bb_rig::Rig;
use crate::ExplorerView;

/// The panel under the editor with one hit whose line is 400 chars wide.
fn show_long_hit(rig: &Rig, vcx: &mut VisualTestContext) {
    let long: String = format!("let hit = {};", "x".repeat(400));
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let path: bb_intel::DocPath = bb_intel::DocPath::new(v.root.join("B.cs")).unwrap();
            let place: crate::intel::bb_worker::protocol::Place =
                crate::intel::bb_worker::protocol::Place {
                    location: bb_intel::Location {
                        uri: bb_intel::DocUri::File(path),
                        span: bb_intel::Span {
                            start: bb_text::LspPoint {
                                line: 0,
                                character: 0,
                            },
                            end: bb_text::LspPoint {
                                line: 0,
                                character: 1,
                            },
                        },
                    },
                    preview: format!("   {long}"),
                };
            v.intel.references = Some(crate::intel::model::popups::PlacesPanel {
                title: String::from("1 result"),
                places: vec![place],
                pick: crate::intel::model::popups::Pick::default(),
            });
            cx.notify();
        },
    );
}

#[gpui::test]
fn the_panel_drags_taller_lists_the_match_first_and_scrolls_sideways(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = split_ab(cx);
    show_long_hit(&rig, vcx);
    vcx.run_until_parked();
    let tall: f32 = f32::from(vcx.debug_bounds("intel-panel").unwrap().size.height);
    drag(
        vcx,
        "bottom-panel-edge",
        gpui::point(gpui::px(0.0), gpui::px(-80.0)),
    );
    let taller: f32 = f32::from(vcx.debug_bounds("intel-panel").unwrap().size.height);
    assert!(
        (taller - tall - 80.0).abs() <= 1.0,
        "panel {tall} -> {taller}"
    );
    let rows: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("intel-rows").unwrap();
    let row: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("place-0").unwrap();
    assert!(
        row.size.width > rows.size.width,
        "the row keeps its full width"
    );
    let path: gpui::Bounds<gpui::Pixels> = vcx.debug_bounds("place-path-0").unwrap();
    assert!(
        path.origin.x > row.origin.x + gpui::px(400.0),
        "the match comes first, its path after it"
    );
    vcx.simulate_event(gpui::ScrollEvent {
        position: rows.center(),
        delta: gpui::ScrollDelta::Pixels(gpui::point(gpui::px(-300.0), gpui::px(0.0))),
        ..Default::default()
    });
    let x: f32 = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        f32::from(v.intel.rows_scroll.offset().x)
    });
    assert!(x < -100.0, "scrolled sideways: {x}");
}

/// Click the element drawn with `selector`.
fn click(vcx: &mut VisualTestContext, selector: &'static str) {
    let at: gpui::Point<gpui::Pixels> = vcx.debug_bounds(selector).unwrap().center();
    vcx.simulate_click(at, gpui::Modifiers::default());
    vcx.run_until_parked();
}

/// BB-BUG-047: every panel under the editor closes on its ✕.
#[gpui::test]
fn find_in_files_results_close_on_their_cross(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = split_ab(cx);
    show_long_hit(&rig, vcx);
    vcx.run_until_parked();
    click(vcx, "intel-panel-close");
    assert!(vcx.debug_bounds("intel-panel").is_none(), "panel gone");
    let left: bool = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.intel.references.is_some() || v.intel.problems_open
    });
    assert!(!left, "nothing left to show");
}

#[gpui::test]
fn the_terminal_closes_on_its_cross_and_the_shell_goes_on(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = split_ab(cx);
    vcx.dispatch_action(crate::terminal::ToggleTerminal);
    vcx.run_until_parked();
    click(vcx, "terminal-close");
    assert!(vcx.debug_bounds("terminal").is_none(), "terminal hidden");
    let shell: bool = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.terminal.session.is_some()
    });
    assert!(shell, "the shell still runs");
    vcx.dispatch_action(crate::terminal::ToggleTerminal);
    vcx.run_until_parked();
    assert!(
        vcx.debug_bounds("terminal").is_some(),
        "Ctrl+` brings it back"
    );
    vcx.dispatch_action(crate::terminal::ToggleTerminal);
}

#[gpui::test]
fn the_debug_panel_hides_on_its_cross_and_comes_back_on_a_pause(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = split_ab(cx);
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            drop(crate::run::bb_debug::session::control::fake_paused(
                v,
                Vec::new(),
                None,
            ));
            cx.notify();
        },
    );
    vcx.run_until_parked();
    click(vcx, "debug-panel-close");
    assert!(
        vcx.debug_bounds("debug-panel").is_none(),
        "debug panel hidden"
    );
    let live: bool = rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.debug.live.is_some()
    });
    assert!(live, "the session goes on");
    vcx.dispatch_action(crate::run::bb_debug::StartOrContinue);
    vcx.run_until_parked();
    assert!(
        vcx.debug_bounds("debug-panel").is_some(),
        "F5 shows it again"
    );
}
