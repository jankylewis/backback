//! One shaped line: the x of every column and the column under every x.
//! The caret, a click, a popup and a hover all read it, so each agrees with
//! the glyphs on screen (BB-UIX-001: never a constant char width).

use gpui::{px, ShapedLine, SharedString, TextRun, Window};

/// The caret quad: thin, so it never covers a glyph.
pub(crate) const CARET_W: f32 = 2.0;

/// A line's text laid out exactly as the row paints it.
pub(crate) struct Columns {
    line: ShapedLine,
    text: String,
}

impl Columns {
    /// Shape `text` (one line, no line break) in the editor font.
    pub(crate) fn shape(window: &Window, text: &str, fonts: &bb_ui::Typography) -> Columns {
        let run: TextRun = TextRun {
            len: text.len(),
            font: gpui::font(fonts.family.clone()),
            color: gpui::black(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line: ShapedLine = window.text_system().shape_line(
            SharedString::from(text.to_owned()),
            px(fonts.size()),
            &[run],
            None,
        );
        Columns {
            line,
            text: text.to_owned(),
        }
    }

    /// The x, from the text's left edge, of the boundary before `column`.
    pub(crate) fn x_of(&self, column: usize) -> f32 {
        let byte: usize = self
            .text
            .char_indices()
            .nth(column)
            .map_or(self.text.len(), |(byte, _): (usize, char)| byte);
        f32::from(self.line.x_for_index(byte))
    }

    /// The column whose boundary is nearest to `x` (from the text's left edge).
    pub(crate) fn column_at(&self, x: f32) -> usize {
        let mut byte: usize = self.line.closest_index_for_x(px(x.max(0.0)));
        // gpui returns the line end for any x past the LAST glyph's start, so
        // a click on the left half of the last char would land after it.
        if byte == self.text.len() {
            if let Some((last, _)) = self.text.char_indices().last() {
                let start: f32 = f32::from(self.line.x_for_index(last));
                if x < f32::midpoint(start, f32::from(self.line.width)) {
                    byte = last;
                }
            }
        }
        self.text
            .char_indices()
            .take_while(|(start, _): &(usize, char)| *start < byte)
            .count()
    }
}
