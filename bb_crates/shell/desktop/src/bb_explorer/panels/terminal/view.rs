//! The terminal panel: one shaped Menlo line per row and a block cursor;
//! keys go to the shell, Cmd+V pastes. The grid follows the panel's size.

use gpui::{div, prelude::*, px, AnyElement, Context, KeyDownEvent, Window};

use super::bb_session::Screen;
use super::keys::{bytes, Mods};
use super::HEIGHT;
use crate::{color, ExplorerView};

/// Rows and columns that fit, from the font and the editor area's width.
fn grid(view: &ExplorerView, window: &Window, fonts: &bb_ui::Typography) -> (usize, usize) {
    let width: f32 = f32::from(window.viewport_size().width) - fonts.editor_x(view.panel_width)
        + fonts.gutter_width();
    let cell: f32 = crate::editor::Columns::shape(window, "M", fonts)
        .x_of(1)
        .max(1.0);
    (
        fit(width - 16.0, cell),
        fit(HEIGHT - 12.0, fonts.line_height()),
    )
}

/// How many `step`s fit in `space` (no float-to-int casts).
fn fit(space: f32, step: f32) -> usize {
    let mut count: usize = 0;
    let mut used: f32 = step;
    while used <= space && count < 1_000 {
        count += 1;
        used += step;
    }
    count.max(1)
}

fn key(
    view: &mut ExplorerView,
    event: &KeyDownEvent,
    _: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(session): Option<&super::Session> = view.terminal.session.as_ref() else {
        return;
    };
    let stroke: &gpui::Keystroke = &event.keystroke;
    let screen_modes: Screen = session.screen();
    if stroke.modifiers.platform && stroke.key == "v" {
        if let Some(text) = cx
            .read_from_clipboard()
            .and_then(|item: gpui::ClipboardItem| item.text())
        {
            let mut out: Vec<u8> = Vec::new();
            if screen_modes.bracketed_paste {
                out.extend_from_slice(b"\x1b[200~");
            }
            out.extend_from_slice(text.replace('\n', "\r").as_bytes());
            if screen_modes.bracketed_paste {
                out.extend_from_slice(b"\x1b[201~");
            }
            session.input(out);
            cx.stop_propagation();
        }
        return;
    }
    let mods: Mods = Mods {
        ctrl: stroke.modifiers.control,
        alt: stroke.modifiers.alt,
        cmd: stroke.modifiers.platform,
    };
    if let Some(out) = bytes(
        &stroke.key,
        stroke.key_char.as_deref(),
        mods,
        screen_modes.app_cursor,
    ) {
        session.input(out);
        cx.stop_propagation();
    }
}

fn row(text: String, cursor: Option<usize>, theme: bb_ui::Theme, height: f32) -> gpui::Div {
    let base: gpui::Div = div().h(px(height)).flex().flex_none().whitespace_nowrap();
    let Some(at): Option<usize> = cursor else {
        return base.child(text);
    };
    let chars: Vec<char> = text.chars().collect();
    let before: String = chars.iter().take(at).collect();
    let under: String = chars
        .get(at)
        .map_or_else(|| " ".to_owned(), char::to_string);
    let after: String = chars.iter().skip(at.saturating_add(1)).collect();
    let pad: String = " ".repeat(at.saturating_sub(chars.len()));
    base.child(format!("{before}{pad}"))
        .child(
            div()
                .debug_selector(|| "terminal-cursor".to_owned())
                .bg(color(theme.text))
                .text_color(color(theme.bg))
                .child(under),
        )
        .child(after)
}

/// The panel when open; also fits the shell's grid to it.
pub(crate) fn panel(
    view: &mut ExplorerView,
    window: &Window,
    cx: &mut Context<ExplorerView>,
) -> Option<AnyElement> {
    if !view.terminal.open {
        return None;
    }
    let fonts: bb_ui::Typography = crate::settings::typography(cx);
    let (columns, lines): (usize, usize) = grid(view, window, &fonts);
    let session: &mut super::Session = view.terminal.session.as_mut()?;
    session.resize(columns, lines);
    let screen: Screen = session.screen();
    let theme: bb_ui::Theme = view.theme;
    let height: f32 = fonts.line_height();
    let rows: Vec<gpui::Div> = screen
        .rows
        .into_iter()
        .enumerate()
        .map(|(index, text): (usize, String)| {
            let at: Option<usize> = screen
                .cursor
                .filter(|(line, _): &(usize, usize)| *line == index)
                .map(|(_, column): (usize, usize)| column);
            row(text, at, theme, height)
        })
        .collect();
    Some(
        div()
            .id("terminal")
            .debug_selector(|| "terminal".to_owned())
            .track_focus(&view.terminal.focus)
            .on_key_down(cx.listener(key))
            .h(px(HEIGHT))
            .flex_none()
            .p(px(6.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(color(theme.bg))
            .border_t_1()
            .border_color(color(theme.border))
            .children(rows)
            .into_any_element(),
    )
}

#[cfg(test)]
mod tests {
    use crate::intel::glue::window_tests::bb_rig::{rig, Rig};
    use crate::ExplorerView;
    use gpui::{TestAppContext, VisualTestContext};

    fn rows(rig: &Rig, vcx: &mut VisualTestContext) -> Vec<String> {
        rig.view.read_with(vcx, |v: &ExplorerView, _: &gpui::App| {
            v.terminal
                .session
                .as_ref()
                .map(|s: &super::super::Session| s.screen().rows)
                .unwrap_or_default()
        })
    }

    /// Ctrl+backtick opens the shell; keys typed in the window run there and
    /// the output is on the panel's screen.
    #[gpui::test]
    fn keys_typed_in_the_window_run_in_the_shell(cx: &mut TestAppContext) {
        let (rig, vcx): (Rig, &mut VisualTestContext) = rig(cx, false);
        vcx.dispatch_action(super::super::ToggleTerminal);
        vcx.run_until_parked();
        assert!(rig
            .view
            .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.terminal.open));
        vcx.simulate_keystrokes("e c h o space b b 4 2 o k enter");
        let mut seen: bool = false;
        for _ in 0..500 {
            vcx.run_until_parked();
            if rows(&rig, vcx)
                .iter()
                .any(|row: &String| row.contains("bb42ok"))
            {
                seen = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(seen, "{:?}", rows(&rig, vcx));
        vcx.dispatch_action(super::super::ToggleTerminal);
        assert!(!rig
            .view
            .read_with(vcx, |v: &ExplorerView, _: &gpui::App| v.terminal.open));
    }
}
