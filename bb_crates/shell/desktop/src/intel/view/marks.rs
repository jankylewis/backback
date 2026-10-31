//! Which chars of a line carry a diagnostic, for squiggles and the gutter.

use bb_intel::{Diagnostic, Severity};
use bb_text::{CharIdx, Document, LspPoint};

/// How a mark is drawn, worst first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Ink {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// Information, a hint, or no severity.
    Faint,
}

impl Ink {
    pub(crate) fn of(severity: Option<Severity>) -> Self {
        match severity {
            Some(Severity::Error) => Self::Error,
            Some(Severity::Warning) => Self::Warning,
            Some(Severity::Information | Severity::Hint) | None => Self::Faint,
        }
    }
}

/// A squiggle on one line, in char columns, end exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Mark {
    pub(crate) from: usize,
    pub(crate) to: usize,
    pub(crate) ink: Ink,
}

/// The squiggles of `line`. An empty range still marks one char, so a
/// missing `;` shows.
pub(crate) fn line_marks(doc: &Document, diagnostics: &[Diagnostic], line: usize) -> Vec<Mark> {
    let Ok(wanted): Result<u32, std::num::TryFromIntError> = u32::try_from(line) else {
        return Vec::new();
    };
    let Some(start): Option<CharIdx> = doc.line_start(line) else {
        return Vec::new();
    };
    let width: usize = doc.line(line).map_or(0, |text: String| {
        text.trim_end_matches(['\r', '\n']).chars().count()
    });
    diagnostics
        .iter()
        .filter(|d: &&Diagnostic| d.span.start.line <= wanted && wanted <= d.span.end.line)
        .map(|d: &Diagnostic| {
            let from: usize = if d.span.start.line == wanted {
                column(doc, start, width, d.span.start)
            } else {
                0
            };
            let to: usize = if d.span.end.line == wanted {
                column(doc, start, width, d.span.end)
            } else {
                width
            };
            let to: usize = if to <= from {
                from.saturating_add(1)
            } else {
                to
            };
            Mark {
                from,
                to,
                ink: Ink::of(d.severity),
            }
        })
        .collect()
}

/// The char column of `point` on the line starting at `start`, at most
/// `width`.
fn column(doc: &Document, start: CharIdx, width: usize, point: LspPoint) -> usize {
    doc.char_of_lsp(point)
        .map_or(0, |at: CharIdx| at.raw().saturating_sub(start.raw()))
        .min(width)
}

/// The worst ink among `marks`, for the gutter dot.
pub(crate) fn worst(marks: &[Mark]) -> Option<Ink> {
    marks.iter().map(|m: &Mark| m.ink).min()
}

/// The worst mark covering char `column`, if any.
pub(crate) fn at(marks: &[Mark], column: usize) -> Option<Ink> {
    marks
        .iter()
        .filter(|m: &&Mark| m.from <= column && column < m.to)
        .map(|m: &Mark| m.ink)
        .min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_intel::Span;

    fn diag(from: (u32, u32), to: (u32, u32), severity: Severity) -> Diagnostic {
        Diagnostic {
            span: Span {
                start: LspPoint {
                    line: from.0,
                    character: from.1,
                },
                end: LspPoint {
                    line: to.0,
                    character: to.1,
                },
            },
            severity: Some(severity),
            code: None,
            message: String::new(),
        }
    }

    #[test]
    fn squiggles_follow_utf16_columns_span_lines_and_never_vanish_when_empty() {
        let doc: Document = Document::from_text("a😀bad\r\nnext\nend", None);
        let list: Vec<Diagnostic> = vec![
            diag((0, 3), (0, 6), Severity::Warning),
            diag((0, 5), (2, 1), Severity::Error),
            diag((2, 3), (2, 3), Severity::Hint),
        ];
        let top: Vec<Mark> = line_marks(&doc, &list, 0);
        assert_eq!((top[0].from, top[0].to), (2, 5));
        assert_eq!((top[1].from, top[1].to), (4, 5));
        assert_eq!(
            line_marks(&doc, &list, 1)[0],
            Mark {
                from: 0,
                to: 4,
                ink: Ink::Error
            }
        );
        let bottom: Vec<Mark> = line_marks(&doc, &list, 2);
        assert_eq!((bottom[1].from, bottom[1].to), (3, 4));
        assert_eq!(worst(&top), Some(Ink::Error));
        assert_eq!(at(&top, 2), Some(Ink::Warning));
        assert_eq!(at(&top, 4), Some(Ink::Error));
        assert_eq!(at(&top, 0), None);
    }
}
