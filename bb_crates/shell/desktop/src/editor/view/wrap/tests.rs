use super::bb_rows::{breaks, Row, Rows};
use proptest::prelude::*;

fn unit(_: char) -> f32 {
    1.0
}

fn parts(text: &str, width: f32) -> Vec<String> {
    breaks(0, text, width, &mut unit)
        .iter()
        .map(|row: &Row| {
            text.chars()
                .skip(row.start)
                .take(row.end - row.start)
                .collect()
        })
        .collect()
}

#[test]
fn a_line_breaks_after_the_last_space_that_fits() {
    assert_eq!(
        parts("Left to do: a 503 error", 12.0),
        ["Left to do: ", "a 503 error"]
    );
    assert_eq!(parts("short", 12.0), ["short"]);
    assert_eq!(parts("", 12.0), [""]);
}

#[test]
fn a_word_wider_than_the_row_breaks_mid_word() {
    assert_eq!(parts("abcdefghij", 4.0), ["abcd", "efgh", "ij"]);
}

#[test]
fn later_rows_start_at_the_line_indent_as_in_zed() {
    let rows: Vec<Row> = breaks(3, "  aaa bbb ccc", 8.0, &mut unit);
    assert_eq!(rows.len(), 3);
    assert_eq!(
        (rows[0].indent, rows[1].indent, rows[1].first),
        (0.0, 2.0, false)
    );
    assert!(rows[0].first && !rows[0].last && rows[2].last);
    assert!(rows.iter().all(|row: &Row| row.line == 3));
    let deep: Vec<Row> = breaks(0, "        aaaa bbbb", 10.0, &mut unit);
    assert_eq!(
        deep[1].indent, 0.0,
        "an indent past half the row is dropped"
    );
}

#[test]
fn row_of_puts_a_break_column_on_the_next_row_and_the_end_on_the_last() {
    let rows: Rows = Rows::build(["one", "abcd efgh ijkl", ""].into_iter(), 5.0, &mut unit);
    assert_eq!(rows.len(), 5);
    assert_eq!(rows.row_of(0, 3), 0);
    assert_eq!(rows.row_of(1, 0), 1);
    assert_eq!(rows.row_of(1, 4), 1, "the space hangs on its row");
    assert_eq!(rows.row_of(1, 5), 2, "a break column starts the next row");
    assert_eq!(rows.row_of(1, 14), 3, "the line end is on its last row");
    assert_eq!(rows.row_of(2, 0), 4);
    assert_eq!(rows.lines_of(&(1..4)), 1..2);
    assert_eq!(rows.lines_of(&(0..5)), 0..3);
}

proptest! {
    /// Rows cover every char of every line once, in order; a row only
    /// overflows by its hanging spaces; every column maps to a row that
    /// holds it.
    #[test]
    fn rows_cover_each_line_and_fit(
        lines in proptest::collection::vec("[a-z ]{0,60}", 0..8),
        width in 1u8..30,
    ) {
        let width: f32 = f32::from(width);
        let rows: Rows = Rows::build(lines.iter().map(String::as_str), width, &mut unit);
        let mut index: usize = 0;
        for (line, text) in lines.iter().enumerate() {
            let mut next: usize = 0;
            while let Some(row) = rows.get(index).filter(|row: &Row| row.line == line) {
                prop_assert_eq!(row.start, next);
                prop_assert!(row.end > row.start || (row.first && row.last));
                let part: String = text.chars().skip(row.start).take(row.end - row.start).collect();
                let room: f32 = width - row.indent;
                let used: usize = part.trim_end_matches(' ').chars().count();
                prop_assert!(used <= 1 || f32::from(u16::try_from(used).unwrap()) <= room, "{:?} in {}", part, room);
                next = row.end;
                index += 1;
            }
            prop_assert_eq!(next, text.chars().count());
            for column in 0..=next {
                let row: Row = rows.get(rows.row_of(line, column)).unwrap();
                prop_assert_eq!(row.line, line);
                prop_assert!(row.start <= column && (column < row.end || row.last || column == 0));
            }
        }
        prop_assert_eq!(index, rows.len());
    }
}
