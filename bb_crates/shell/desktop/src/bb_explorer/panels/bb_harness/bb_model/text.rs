//! Large text cut into bounded display rows, so a view shapes only what it shows.

use std::ops::Range;

/// Longest row in characters; longer lines wrap onto more rows.
pub(crate) const ROW_CHARS: usize = 160;

#[derive(Clone, Debug, Default)]
pub(crate) struct RowText {
    text: String,
    rows: Vec<Range<usize>>,
}

impl RowText {
    pub(crate) fn rows(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn row(&self, index: usize) -> &str {
        self.rows
            .get(index)
            .and_then(|range: &Range<usize>| self.text.get(range.clone()))
            .unwrap_or_default()
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }
}

impl From<String> for RowText {
    fn from(text: String) -> Self {
        let rows: Vec<Range<usize>> = split_rows(&text);
        Self { text, rows }
    }
}

fn split_rows(text: &str) -> Vec<Range<usize>> {
    let mut rows: Vec<Range<usize>> = Vec::new();
    let mut offset: usize = 0;
    for line in text.split('\n') {
        let mut start: usize = offset;
        let mut count: usize = 0;
        for (index, _) in line.char_indices() {
            if count == ROW_CHARS {
                let cut: usize = offset.saturating_add(index);
                rows.push(start..cut);
                start = cut;
                count = 0;
            }
            count = count.saturating_add(1);
        }
        let end: usize = offset.saturating_add(line.len());
        rows.push(start..end);
        offset = end.saturating_add(1);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::{RowText, ROW_CHARS};
    use proptest::prelude::*;

    #[test]
    fn long_line_wraps_into_bounded_rows_and_lines_stay_separate() {
        let text: RowText = RowText::from(format!("{}\nab\n", "é".repeat(ROW_CHARS * 2 + 1)));
        assert_eq!(text.rows(), 5);
        assert_eq!(text.row(0).chars().count(), ROW_CHARS);
        assert_eq!(text.row(2), "é");
        assert_eq!(text.row(3), "ab");
        assert_eq!(text.row(4), "");
        assert_eq!(text.row(5), "");
    }

    proptest! {
        #[test]
        fn rows_cover_the_text_in_order_without_newlines(text in "(\\PC|\n){0,600}") {
            let rows: RowText = RowText::from(text.clone());
            let mut cursor: usize = 0;
            for range in &rows.rows {
                let gap: &str = &text[cursor..range.start];
                prop_assert!(gap.is_empty() || gap == "\n");
                let row: &str = &text[range.clone()];
                prop_assert!(!row.contains('\n'));
                prop_assert!(row.chars().count() <= ROW_CHARS);
                cursor = range.end;
            }
            prop_assert_eq!(cursor, text.len());
        }
    }
}
