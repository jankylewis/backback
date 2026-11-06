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

/// The shell under a PTY, without a window (BBEOD11).
mod session {
    use super::super::Session;
    use std::time::{Duration, Instant};
    use tokio::sync::mpsc::UnboundedSender;

    fn wait_for(session: &Session, needle: &str, within: Duration) -> bool {
        let started: Instant = Instant::now();
        while started.elapsed() < within {
            if session
                .screen()
                .rows
                .iter()
                .any(|row: &String| row.contains(needle))
            {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    /// A real shell under a real PTY: typed input runs, output is parsed.
    #[test]
    fn a_typed_command_runs_and_its_output_reaches_the_screen() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let (wake, _woken): (UnboundedSender<()>, _) = tokio::sync::mpsc::unbounded_channel();
        let mut session: Session = Session::start(dir.path(), 80, 12, wake).unwrap();
        session.input(b"echo bb_$((40+2))_ok\r".to_vec());
        assert!(
            wait_for(&session, "bb_42_ok", Duration::from_secs(10)),
            "{:?}",
            session.screen().rows
        );
        session.resize(40, 6);
        assert_eq!(session.screen().rows.len(), 6);
    }

    /// Heavy output does not block: 100,000 lines and the prompt is back fast.
    #[test]
    fn a_hundred_thousand_lines_stream_through_quickly() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let (wake, _woken): (UnboundedSender<()>, _) = tokio::sync::mpsc::unbounded_channel();
        let session: Session = Session::start(dir.path(), 100, 20, wake).unwrap();
        let started: Instant = Instant::now();
        session.input(b"seq 1 100000; echo bb_done_$((1+1))\r".to_vec());
        assert!(
            wait_for(&session, "bb_done_2", Duration::from_secs(20)),
            "{:?}",
            session.screen().rows
        );
        let took: Duration = started.elapsed();
        eprintln!("terminal_100k_lines_ms={}", took.as_millis());
        assert!(took < Duration::from_secs(10), "{took:?}");
    }

    /// BB-BUG-053: a selection over echoed output reads back as that text,
    /// shows as a column range on its row, and a clear drops it.
    #[test]
    fn a_selection_reads_back_the_selected_text() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let (wake, _woken): (UnboundedSender<()>, _) = tokio::sync::mpsc::unbounded_channel();
        let session: Session = Session::start(dir.path(), 80, 12, wake).unwrap();
        session.input(b"echo bb_sel_$((1+1))ok\r".to_vec());
        assert!(wait_for(&session, "bb_sel_2ok", Duration::from_secs(10)));
        let row: usize = session
            .screen()
            .rows
            .iter()
            .position(|r: &String| r.starts_with("bb_sel_2ok"))
            .unwrap();
        session.select_from(cell(row, 0, false));
        session.select_to(cell(row, 9, true));
        assert_eq!(session.selected_text().as_deref(), Some("bb_sel_2ok"));
        assert_eq!(
            session.screen().selected.get(row).copied().flatten(),
            Some((0, 9))
        );
        session.clear_selection();
        assert_eq!(session.selected_text(), None);
    }

    /// BB-BUG-058: in a full-screen app (the alternate screen: less, vim)
    /// the wheel sends Up keys, as VS Code does; the app sees them.
    #[test]
    fn in_a_full_screen_app_the_wheel_sends_arrow_keys() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let (wake, _woken): (UnboundedSender<()>, _) = tokio::sync::mpsc::unbounded_channel();
        let session: Session = Session::start(dir.path(), 80, 12, wake).unwrap();
        session.input(b"printf '\\033[?1049h'; echo bb_alt_$((1+1)); cat -v\r".to_vec());
        assert!(wait_for(&session, "bb_alt_2", Duration::from_secs(10)));
        session.scroll(2);
        assert!(
            wait_for(&session, "^[[A^[[A", Duration::from_secs(10)),
            "{:?}",
            session.screen().rows
        );
        session.input(vec![3]);
    }

    fn cell(row: usize, column: usize, right_half: bool) -> super::super::Cell {
        super::super::Cell {
            row,
            column,
            right_half,
        }
    }
}
