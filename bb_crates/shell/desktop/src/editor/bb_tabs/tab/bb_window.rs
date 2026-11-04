//! Large files: the tree covers only lines around the view. A full tree-sitter
//! tree costs about 41 bytes per source byte (862 MB for a 20 MB file).

use std::ops::Range;

use bb_text::{BytePoint, Document};

use super::Tab;

/// Above this size only a window of lines around the view is parsed.
pub(crate) const LARGE_FILE_BYTES: usize = 4 * 1024 * 1024;

/// Lines parsed above and below the visible lines.
const MARGIN_LINES: usize = 1_000;

/// The window to parse at load: the top of a large file, `None` otherwise.
pub(super) fn first_window(document: &Document) -> Option<Range<usize>> {
    (document.rope().len_bytes() > LARGE_FILE_BYTES).then(|| 0..MARGIN_LINES.saturating_mul(2))
}

/// Byte points of a line window, for the parser.
pub(super) fn points(
    document: &Document,
    lines: Option<&Range<usize>>,
) -> Option<(BytePoint, BytePoint)> {
    let lines: &Range<usize> = lines?;
    Some((
        document.line_point(lines.start)?,
        document.line_point(lines.end)?,
    ))
}

impl Tab {
    /// The byte window for the next parse; `None` parses the whole file.
    pub(crate) fn parse_window(&self) -> Option<(BytePoint, BytePoint)> {
        points(&self.document, self.window_lines.as_ref())
    }

    /// Record the lines on screen. Returns true when they left the parsed
    /// window: the window moves and a parse is due.
    pub(crate) fn show_lines(&mut self, visible: &Range<usize>) -> bool {
        let Some(current): Option<&Range<usize>> = self.window_lines.as_ref() else {
            return false;
        };
        let slack: usize = MARGIN_LINES / 4;
        let last: usize = self.document.len_lines();
        let low: bool = current.start > 0 && visible.start < current.start.saturating_add(slack);
        let high: bool = current.end < last && visible.end.saturating_add(slack) > current.end;
        if !low && !high {
            return false;
        }
        self.window_lines = Some(
            visible.start.saturating_sub(MARGIN_LINES)
                ..visible.end.saturating_add(MARGIN_LINES).min(last),
        );
        self.live.mark_stale();
        true
    }
}
