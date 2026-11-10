//! BB-BUG-062 through the window: more tabs than the bar can show. The
//! active tab scrolls into sight at full width, and the wheel scrolls the
//! bar back to the first tab.

use gpui::VisualTestContext;
use gpui::{point, px, Bounds, Pixels, ScrollDelta, ScrollEvent, TestAppContext};

use crate::intel::glue::window_tests::bb_rig::{rig_with, Rig};
use crate::ExplorerView;

/// The rig's `A.cs` plus `count` more tabs with long names; the last active.
fn many_tabs(cx: &mut TestAppContext, count: usize) -> (Rig, &mut VisualTestContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig_with(cx, false, true, "class A {}\n");
    for n in 1..=count {
        let path: std::path::PathBuf = rig.root.join(format!("ImsStandaloneDownload{n:02}.cs"));
        std::fs::write(&path, "class B {}\n").unwrap();
        rig.view.update(
            vcx,
            |v: &mut ExplorerView, cx: &mut gpui::Context<ExplorerView>| {
                v.tabs.open(&path).unwrap();
                cx.notify();
            },
        );
        vcx.run_until_parked();
    }
    (rig, vcx)
}

fn bounds(vcx: &mut VisualTestContext, selector: String) -> Bounds<Pixels> {
    let key: &'static str = Box::leak(selector.into_boxed_str());
    vcx.debug_bounds(key).unwrap_or_else(|| panic!("no {key}"))
}

/// Whether `tab` lies whole inside the bar, left to right.
fn whole_in_bar(vcx: &mut VisualTestContext, tab: usize) -> bool {
    let bar: Bounds<Pixels> = bounds(vcx, String::from("tab-bar-0"));
    let item: Bounds<Pixels> = bounds(vcx, format!("tab-{tab}"));
    item.left() >= bar.left() - px(0.5) && item.right() <= bar.right() + px(0.5)
}

#[gpui::test]
fn the_active_tab_scrolls_into_sight_and_the_wheel_brings_the_first_back(cx: &mut TestAppContext) {
    let count: usize = 12;
    let (_rig, vcx): (Rig, &mut VisualTestContext) = many_tabs(cx, count);
    let first: Bounds<Pixels> = bounds(vcx, String::from("tab-1"));
    let last: Bounds<Pixels> = bounds(vcx, format!("tab-{count}"));
    assert!(
        (last.size.width - first.size.width).abs() < px(1.0),
        "tabs keep their width: {first:?} vs {last:?}"
    );
    assert!(whole_in_bar(vcx, count), "the active last tab is in sight");
    assert!(!whole_in_bar(vcx, 0), "the bar scrolled past the first tab");
    let bar: Bounds<Pixels> = bounds(vcx, String::from("tab-bar-0"));
    for _ in 0..40 {
        vcx.simulate_event(ScrollEvent {
            position: bar.center(),
            delta: ScrollDelta::Pixels(point(px(0.0), px(200.0))),
            ..ScrollEvent::default()
        });
    }
    vcx.run_until_parked();
    assert!(whole_in_bar(vcx, 0), "a mouse wheel scrolls the bar back");
}
