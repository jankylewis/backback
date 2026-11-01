use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::super::model::RowText;
use super::super::view::status_badge;
use super::super::{Harness, ServerStatus};
use bb_ui::bb_theme::{Theme, DARK, LIGHT};
use bb_ui::Rgb;
use gpui::{px, size, Bounds, TestAppContext, VisualTestContext, WindowBounds, WindowOptions};

fn harness_window(
    cx: &mut TestAppContext,
) -> (gpui::Entity<crate::ExplorerView>, VisualTestContext) {
    let fixture: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/_Harness");
    let window: gpui::WindowHandle<crate::ExplorerView> = cx.update(|cx: &mut gpui::App| {
        let tree: bb_explorer::Explorer = bb_explorer::Explorer::open(&fixture).unwrap();
        let harness: Harness = Harness::load(&fixture).unwrap();
        let bounds: Bounds<gpui::Pixels> = Bounds::centered(None, size(px(1000.), px(700.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window: &mut gpui::Window, cx: &mut gpui::App| {
                crate::launch::new_view(tree, harness, window.appearance(), cx)
            },
        )
        .unwrap()
    });
    let mut visual: VisualTestContext = VisualTestContext::from_window(window.into(), cx);
    let view: gpui::Entity<crate::ExplorerView> = window.root(&mut visual).unwrap();
    (view, visual)
}

fn ten_megabyte_result() -> String {
    let text: String = "x".repeat(10 * 1024 * 1024);
    serde_json::to_string_pretty(&serde_json::json!([{"type":"text","text":text}])).unwrap()
}

fn frame_time(
    view: &gpui::Entity<crate::ExplorerView>,
    visual: &mut VisualTestContext,
) -> Duration {
    let started: Instant = Instant::now();
    view.update_in(
        visual,
        |_: &mut crate::ExplorerView,
         window: &mut gpui::Window,
         cx: &mut gpui::Context<crate::ExplorerView>| {
            cx.notify();
            window.refresh();
        },
    );
    visual.run_until_parked();
    started.elapsed()
}

#[gpui::test]
fn ten_megabyte_result_paints_within_one_second_and_redraws_within_50_ms(cx: &mut TestAppContext) {
    let (view, mut visual): (gpui::Entity<crate::ExplorerView>, VisualTestContext) =
        harness_window(cx);
    let result: String = ten_megabyte_result();
    let arrived: Instant = Instant::now();
    let rows: RowText = RowText::from(result);
    let built: Duration = arrived.elapsed();
    view.update_in(
        &mut visual,
        |state: &mut crate::ExplorerView,
         _: &mut gpui::Window,
         _: &mut gpui::Context<crate::ExplorerView>| {
            state.harness.selected = Some(0);
            state.harness.servers[0].result = Some(rows);
        },
    );
    let first: Duration = built + frame_time(&view, &mut visual);
    let redraw: Duration = frame_time(&view, &mut visual);
    println!("10 MB result: rows built {built:?}, first paint {first:?}, redraw {redraw:?}");
    assert!(first < Duration::from_secs(1), "first paint {first:?}");
    assert!(redraw < Duration::from_millis(50), "redraw {redraw:?}");
}

fn is_green(colour: Rgb) -> bool {
    colour.g > colour.r.saturating_add(16) && colour.g > colour.b.saturating_add(16)
}

#[test]
fn status_badges_are_distinct_per_state_and_running_is_green_in_both_themes() {
    let states: [ServerStatus; 5] = [
        ServerStatus::Stopped,
        ServerStatus::Starting,
        ServerStatus::Stopping,
        ServerStatus::Running,
        ServerStatus::Crashed("exit status: 3".into()),
    ];
    for theme in [DARK, LIGHT] {
        let badges: Vec<(&'static str, Rgb)> = states
            .iter()
            .map(|status: &ServerStatus| status_badge(status, theme))
            .collect();
        for (index, badge) in badges.iter().enumerate() {
            for other in badges.iter().skip(index + 1) {
                assert_ne!(badge.0, other.0, "{}", theme.name);
                assert_ne!(
                    badge.1, other.1,
                    "{} {} vs {}",
                    theme.name, badge.0, other.0
                );
            }
        }
        let running: (&'static str, Rgb) = status_badge(&ServerStatus::Running, theme);
        assert!(
            is_green(running.1),
            "{} running {}",
            theme.name,
            running.1.hex()
        );
        assert_eq!(running.1, Theme::for_kind(Some(theme.kind)).ok);
    }
}
