//! Position newtypes. Each one is a different unit; mixing them is a compile error.

/// Index of a Unicode scalar value (a `char`) in a buffer, 0-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharIdx(usize);

impl CharIdx {
    /// Wrap a raw char index. Bounds are checked by the buffer, not here.
    #[must_use]
    pub const fn new(raw: usize) -> Self {
        Self(raw)
    }

    /// The raw index, for the rope API only.
    #[must_use]
    pub const fn raw(self) -> usize {
        self.0
    }
}

/// Index of a line in a buffer, 0-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineIdx(usize);

impl LineIdx {
    /// Wrap a raw line index.
    #[must_use]
    pub const fn new(raw: usize) -> Self {
        Self(raw)
    }

    /// The raw index, for the rope API only.
    #[must_use]
    pub const fn raw(self) -> usize {
        self.0
    }
}

/// A position as LSP sends it: 0-based line and 0-based UTF-16 code-unit column.
///
/// LSP's default `positionEncoding` is `utf-16`; Roslyn speaks it. Keep this type
/// apart from [`CharIdx`] so a UTF-16 column is never used as a char offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LspPoint {
    /// 0-based line.
    pub line: u32,
    /// 0-based column in UTF-16 code units.
    pub character: u32,
}
