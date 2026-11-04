//! Real mouse input for the readback script: a move or a left click at the
//! char the window draws, optionally with Cmd held (BBEOD15).

use gpui::{AsyncWindowContext, MouseMoveEvent, PlatformInput, WeakEntity, Window};

use crate::ExplorerView;

fn keys(cmd: bool) -> gpui::Modifiers {
    if cmd {
        gpui::Modifiers::command()
    } else {
        gpui::Modifiers::default()
    }
}

/// Move the mouse to the middle of char `col` on `line`, as the window draws it.
pub(super) fn hover(
    view: &WeakEntity<ExplorerView>,
    cx: &mut AsyncWindowContext,
    at: (usize, usize),
    cmd: bool,
) {
    let _gone: gpui::Result<()> = cx.update(|window: &mut Window, cx: &mut gpui::App| {
        if let Some(position) = pointer(view, window, cx, at, true) {
            window.dispatch_event(
                PlatformInput::MouseMove(MouseMoveEvent {
                    position,
                    pressed_button: None,
                    modifiers: keys(cmd),
                }),
                cx,
            );
        }
    });
}

/// A real left click on the boundary before char `col` on `line`: it
/// focuses the editor and puts the cursor there, as a person's click does.
pub(super) fn click(
    view: &WeakEntity<ExplorerView>,
    cx: &mut AsyncWindowContext,
    at: (usize, usize),
    cmd: bool,
) {
    let _gone: gpui::Result<()> = cx.update(|window: &mut Window, cx: &mut gpui::App| {
        let Some(position): Option<gpui::Point<gpui::Pixels>> =
            pointer(view, window, cx, at, false)
        else {
            return;
        };
        press(window, cx, position, keys(cmd));
    });
}

/// A real left click on `line` near the window's right edge: past the line's
/// last char, where a person clicks to reach the end of a line (BB-BUG-022).
pub(super) fn click_past(
    view: &WeakEntity<ExplorerView>,
    cx: &mut AsyncWindowContext,
    line: usize,
) {
    let _gone: gpui::Result<()> = cx.update(|window: &mut Window, cx: &mut gpui::App| {
        let Some(at): Option<gpui::Point<gpui::Pixels>> =
            pointer(view, window, cx, (line, 0), false)
        else {
            return;
        };
        let position: gpui::Point<gpui::Pixels> =
            gpui::point(window.viewport_size().width - gpui::px(40.0), at.y);
        press(window, cx, position, keys(false));
    });
}

/// A left button down then up at `position`.
fn press(
    window: &mut Window,
    cx: &mut gpui::App,
    position: gpui::Point<gpui::Pixels>,
    modifiers: gpui::Modifiers,
) {
    let button: gpui::MouseButton = gpui::MouseButton::Left;
    window.dispatch_event(
        PlatformInput::MouseDown(gpui::MouseDownEvent {
            button,
            position,
            modifiers,
            click_count: 1,
            first_mouse: false,
        }),
        cx,
    );
    window.dispatch_event(
        PlatformInput::MouseUp(gpui::MouseUpEvent {
            button,
            position,
            modifiers,
            click_count: 1,
        }),
        cx,
    );
}

/// Where char `col` of `line` is drawn: its middle, or the boundary before it.
fn pointer(
    view: &WeakEntity<ExplorerView>,
    window: &Window,
    cx: &mut gpui::App,
    (line, col): (usize, usize),
    middle: bool,
) -> Option<gpui::Point<gpui::Pixels>> {
    let view: gpui::Entity<ExplorerView> = view.upgrade()?;
    let (left, top, scrolled, text): (f32, f32, f32, String) =
        view.read_with(cx, |v: &ExplorerView, _: &gpui::App| {
            let state: std::cell::Ref<'_, gpui::UniformListScrollState> = v.tabs.scroll.0.borrow();
            let bounds: gpui::Bounds<gpui::Pixels> = state.base_handle.bounds();
            (
                v.editor_left(&crate::settings::typography(cx))
                    + crate::settings::typography(cx).gutter_width(),
                f32::from(bounds.origin.y),
                f32::from(state.base_handle.offset().y),
                v.tabs
                    .active()
                    .and_then(|doc: &bb_text::Document| doc.line(line))
                    .unwrap_or_default(),
            )
        });
    let row: f32 = f32::from(u16::try_from(line).unwrap_or(u16::MAX));
    let height: f32 = crate::settings::typography(cx).line_height();
    let columns: crate::editor::Columns = crate::editor::Columns::shape(
        window,
        text.trim_end_matches(['\r', '\n']),
        &crate::settings::typography(cx),
    );
    let x: f32 = if middle {
        f32::midpoint(columns.x_of(col), columns.x_of(col.saturating_add(1)))
    } else {
        columns.x_of(col)
    };
    Some(gpui::point(
        gpui::px(left + x),
        gpui::px(top + scrolled + height * row + height / 2.0),
    ))
}
