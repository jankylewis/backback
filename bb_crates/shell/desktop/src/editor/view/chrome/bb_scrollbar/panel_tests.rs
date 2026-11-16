//! BB-FEAT-085: Call Stack, Variables, TESTS and EXPLORER keep each row's
//! full width; a bar shows along the bottom only while a row is wider than
//! the panel, and dragging its thumb scrolls the rows across.

use std::path::PathBuf;

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use super::PanelBar;
use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::run::bb_debug::session::Frame;
use crate::ExplorerView;

fn settle(vcx: &mut VisualTestContext) {
    // The bars read the rows' size from the frame before.
    vcx.run_until_parked();
    vcx.update(|window: &mut gpui::Window, _: &mut gpui::App| window.refresh());
    vcx.run_until_parked();
}

/// Drag `which` panel's thumb 60 px right; how far its rows then scrolled.
fn drag_across(
    rig: &Rig,
    vcx: &mut VisualTestContext,
    thumb: &'static str,
    which: PanelBar,
) -> f32 {
    let from: gpui::Point<gpui::Pixels> = vcx.debug_bounds(thumb).expect(thumb).center();
    let to: gpui::Point<gpui::Pixels> = from + gpui::point(gpui::px(60.0), gpui::px(0.0));
    vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::default());
    vcx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        -f32::from(which.scroll(v).offset().x)
    })
}

#[gpui::test]
fn a_long_frame_gives_the_call_stack_a_bar_and_short_variables_none(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    let frame: Frame = Frame {
        id: 1,
        name: format!("Very.Long.{}.Method()", "Namespace".repeat(40)),
        path: None,
        line: 0,
    };
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let _sent: tokio::sync::mpsc::UnboundedReceiver<
                crate::run::bb_debug::session::Command,
            > = crate::run::bb_debug::session::control::fake_paused(v, vec![frame], None);
            cx.notify();
        },
    );
    settle(vcx);
    assert!(
        vcx.debug_bounds("hbar-thumb-stack").is_some(),
        "a frame wider than the column"
    );
    assert!(
        vcx.debug_bounds("hbar-thumb-vars").is_none(),
        "no variable is wide"
    );
    let moved: f32 = drag_across(&rig, vcx, "hbar-thumb-stack", PanelBar::Stack);
    assert!(moved > 60.0, "the rows moved more than the thumb: {moved}");
}

#[gpui::test]
fn a_long_test_name_gives_the_tests_panel_a_bar(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    let long: String = format!("Sample.{}.Spec.Passes", "Namespace".repeat(40));
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            let project: PathBuf = rig.root.join("Tests.csproj");
            v.tests.side = crate::tests_panel::Side::Tests;
            v.tests.scan = crate::tests_panel::Scan::Done;
            v.tests.model.merge(
                &project,
                bb_testkit::SpecList {
                    project: String::from("Tests"),
                    assembly: PathBuf::from("Tests.dll"),
                    specs: vec![bb_testkit::Fqn::new(&long).unwrap()],
                    elapsed: std::time::Duration::ZERO,
                },
            );
            if let Some(tests) = v.tests.model.project_mut(&project) {
                tests.expanded = true;
            }
            cx.notify();
        },
    );
    settle(vcx);
    let moved: f32 = drag_across(&rig, vcx, "hbar-thumb-tests", PanelBar::Tests);
    assert!(moved > 60.0, "{moved}");
}

#[gpui::test]
fn a_narrow_explorer_scrolls_its_names_across(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    settle(vcx);
    assert!(
        vcx.debug_bounds("hbar-thumb-explorer").is_none(),
        "every name fits"
    );
    rig.view.update(
        vcx,
        |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
            v.panel_width = 70.0;
            cx.notify();
        },
    );
    settle(vcx);
    let moved: f32 = drag_across(&rig, vcx, "hbar-thumb-explorer", PanelBar::Explorer);
    assert!(moved > 0.0, "{moved}");
}
