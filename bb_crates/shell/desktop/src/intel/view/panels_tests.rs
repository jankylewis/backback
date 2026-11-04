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
