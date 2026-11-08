//! Word wrap (BBEOD-WRAP): each file line split into screen rows that fit
//! the pane, as Zed and Excel's Wrap Text do. A row breaks after the last
//! space that fits, or mid-word when a word is wider than the row; spaces
//! at a break stay on the row they end. Later rows of a line start at the
//! line's indent, so wrapped code lines up under its first row.

/// One screen row: chars `start..end` (columns) of file line `line`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Row {
    pub(crate) line: usize,
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// Pixels the row's text moves right: the line's indent on a later row.
    pub(crate) indent: f32,
    /// The row a file line starts with: it shows the line number.
    pub(crate) first: bool,
    /// The line's last row: the caret may sit after its last char.
    pub(crate) last: bool,
}

/// Every row of a file, and the first row of each line.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Rows {
    rows: Vec<Row>,
    first: Vec<usize>,
}

/// The x advance of one char in the editor font.
pub(crate) type Advance<'a> = &'a mut dyn FnMut(char) -> f32;

/// The rows of one line's `text` (no line break) in `width` pixels.
pub(crate) fn breaks(line: usize, text: &str, width: f32, advance: Advance<'_>) -> Vec<Row> {
    let chars: Vec<char> = text.chars().collect();
    let widths: Vec<f32> = chars.iter().map(|c: &char| advance(*c)).collect();
    let lead: usize = chars
        .iter()
        .take_while(|c: &&char| **c == ' ' || **c == '\t')
        .count();
    let indent_px: f32 = widths.iter().take(lead).sum();
    // An indent wider than half the row would leave too little for text.
    let indent: f32 = if indent_px * 2.0 <= width {
        indent_px
    } else {
        0.0
    };
    let mut rows: Vec<Row> = Vec::new();
    let mut start: usize = 0;
    loop {
        let room: f32 = if rows.is_empty() {
            width
        } else {
            width - indent
        };
        let end: usize = row_end(&chars, &widths, start, room);
        rows.push(Row {
            line,
            start,
            end,
            indent: if rows.is_empty() { 0.0 } else { indent },
            first: rows.is_empty(),
            last: end >= chars.len(),
        });
        if end >= chars.len() {
            return rows;
        }
        start = end;
    }
}

/// Where the row from `start` ends in `room` px: at least one char, every
/// space that ends it, and after the last space when a word would not fit.
fn row_end(chars: &[char], widths: &[f32], start: usize, room: f32) -> usize {
    let mut x: f32 = 0.0;
    let mut end: usize = start;
    let mut after_space: Option<usize> = None;
    while let (Some(c), Some(w)) = (chars.get(end), widths.get(end)) {
        let space: bool = c.is_whitespace();
        if x + w > room && end > start && !space {
            return after_space.filter(|at: &usize| *at > start).unwrap_or(end);
        }
        x += w;
        end += 1;
        if space {
            after_space = Some(end);
        }
    }
    end
}

impl Rows {
    /// Wrap every line of `lines` (no line breaks) in `width` pixels.
    pub(crate) fn build<S: AsRef<str>>(
        lines: impl Iterator<Item = S>,
        width: f32,
        advance: Advance<'_>,
    ) -> Rows {
        let mut out: Rows = Rows::default();
        for (line, text) in lines.enumerate() {
            out.first.push(out.rows.len());
            out.rows
                .extend(breaks(line, text.as_ref(), width, &mut *advance));
        }
        out
    }

    /// The number of screen rows.
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    /// Screen row `index`.
    pub(crate) fn get(&self, index: usize) -> Option<Row> {
        self.rows.get(index).copied()
    }

    /// The row that shows `column` of `line`: a column at a break is the
    /// next row's first char; the line's end is on its last row.
    pub(crate) fn row_of(&self, line: usize, column: usize) -> usize {
        let Some(first): Option<usize> = self.first.get(line).copied() else {
            return self.rows.len().saturating_sub(1);
        };
        self.rows
            .iter()
            .enumerate()
            .skip(first)
            .take_while(|(_, row): &(usize, &Row)| row.line == line)
            .take_while(|(_, row): &(usize, &Row)| row.first || row.start <= column)
            .last()
            .map_or(first, |(index, _): (usize, &Row)| index)
    }

    /// The file lines that rows `range` show.
    pub(crate) fn lines_of(&self, range: &std::ops::Range<usize>) -> std::ops::Range<usize> {
        let from: usize = self.get(range.start).map_or(0, |row: Row| row.line);
        let to: usize = range
            .end
            .checked_sub(1)
            .and_then(|last: usize| self.get(last))
            .map_or(from, |row: Row| row.line);
        from..to.saturating_add(1)
    }
}
