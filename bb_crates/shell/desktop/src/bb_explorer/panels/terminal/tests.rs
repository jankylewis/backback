//! BB-BUG-046: the terminal reads in both themes, and its top edge drags
//! its height (the shell gets the rows that now fit).

use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};

use crate::intel::glue::window_tests::bb_rig::{rig, Rig};
use crate::ExplorerView;

/// One sRGB channel, linear.
fn lin(v: u8) -> f64 {
    let s: f64 = f64::from(v) / 255.0;
    if s <= 0.039_28 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of one colour.
fn luminance(c: bb_ui::Rgb) -> f64 {
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

#[test]
fn terminal_text_has_readable_contrast_in_both_themes() {
    for theme in [bb_ui::bb_theme::DARK, bb_ui::bb_theme::LIGHT] {
        let (text, bg): (bb_ui::Rgb, bb_ui::Rgb) = super::view::colors(theme);
        let (a, b): (f64, f64) = (luminance(text), luminance(bg));
        let ratio: f64 = (a.max(b) + 0.05) / (a.min(b) + 0.05);
        assert!(
            ratio >= 4.5,
            "{}: contrast {ratio:.1} below 4.5",
            theme.name
        );
    }
}

fn lines(rig: &Rig, vcx: &mut VisualTestContext) -> usize {
    rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
        v.terminal
            .session
            .as_ref()
            .map_or(0, |s: &super::Session| s.screen().rows.len())
    })
}

#[gpui::test]
fn the_terminal_top_edge_drags_taller_and_the_shell_gets_more_rows(cx: &mut TestAppContext) {
    let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
    vcx.dispatch_action(super::ToggleTerminal);
    vcx.run_until_parked();
    let tall: f32 = f32::from(vcx.debug_bounds("terminal").unwrap().size.height);
    let before: usize = lines(&rig, vcx);
    let from: gpui::Point<gpui::Pixels> = vcx.debug_bounds("terminal-edge").unwrap().center();
    let to: gpui::Point<gpui::Pixels> = from - gpui::point(gpui::px(0.0), gpui::px(100.0));
    vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    vcx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::default());
    vcx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    vcx.run_until_parked();
    let taller: f32 = f32::from(vcx.debug_bounds("terminal").unwrap().size.height);
    assert!(
        (taller - tall - 100.0).abs() <= 1.0,
        "terminal {tall} -> {taller}"
    );
    assert!(
        lines(&rig, vcx) > before,
        "rows {before} -> {}",
        lines(&rig, vcx)
    );
    vcx.dispatch_action(super::ToggleTerminal);
}
