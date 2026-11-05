//! Theme tokens. Dark is the default; light is always available.
//!
//! Palette values are inspired by Rosé Pine (MIT), chosen for low-contrast
//! surfaces with high-contrast text, which keeps long sessions comfortable.

/// An sRGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
}

impl Rgb {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// `#rrggbb` for CSS-like consumers and logs.
    #[must_use]
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Dark or light.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    /// Default.
    Dark,
    /// Opt-in or OS preference.
    Light,
}

/// Semantic colour tokens. Widgets use these names, never raw colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Human name.
    pub name: &'static str,
    /// Kind.
    pub kind: ThemeKind,
    /// Window background.
    pub bg: Rgb,
    /// Panels, sidebars.
    pub surface: Rgb,
    /// Popups, hover.
    pub overlay: Rgb,
    /// Dividers.
    pub border: Rgb,
    /// Primary text.
    pub text: Rgb,
    /// Secondary text.
    pub text_dim: Rgb,
    /// Explorer entries git ignores (BBEOD09): pink, softer than `error`.
    pub ignored: Rgb,
    /// The chosen sidebar tab, EXPLORER or TESTS: the same pink.
    pub chosen: Rgb,
    /// Focus, links.
    pub accent: Rgb,
    /// Selected text's background (editor, terminal): `text` on it reads at
    /// 4.5:1 or more (BB-BUG-054; the accent read at 1.3:1).
    pub selection: Rgb,
    /// Errors.
    pub error: Rgb,
    /// Warnings.
    pub warn: Rgb,
    /// Success, passing tests.
    pub ok: Rgb,
    /// Keywords.
    pub keyword: Rgb,
    /// Strings.
    pub string: Rgb,
    /// Comments.
    pub comment: Rgb,
    /// Types.
    pub ty: Rgb,
    /// C# file icon.
    pub icon_csharp: Rgb,
    /// Markdown file icon.
    pub icon_markdown: Rgb,
    /// Folder icon.
    pub icon_folder: Rgb,
    /// Project and solution icons.
    pub icon_project: Rgb,
    /// Generic file icon.
    pub icon_file: Rgb,
}

/// The default theme.
pub const DARK: Theme = Theme {
    name: "backback dark",
    kind: ThemeKind::Dark,
    bg: Rgb::new(0x23, 0x21, 0x36),
    surface: Rgb::new(0x2a, 0x27, 0x3f),
    overlay: Rgb::new(0x39, 0x35, 0x52),
    border: Rgb::new(0x6e, 0x6a, 0x86),
    text: Rgb::new(0xe0, 0xde, 0xf4),
    text_dim: Rgb::new(0x90, 0x8c, 0xaa),
    ignored: Rgb::new(0xf5, 0xa9, 0xc8),
    chosen: Rgb::new(0xf5, 0xa9, 0xc8),
    accent: Rgb::new(0x9c, 0xcf, 0xd8),
    selection: Rgb::new(0x33, 0x44, 0x6b),
    error: Rgb::new(0xeb, 0x6f, 0x92),
    warn: Rgb::new(0xf6, 0xc1, 0x77),
    ok: Rgb::new(0xa6, 0xda, 0x95),
    keyword: Rgb::new(0x3e, 0x8f, 0xb0),
    string: Rgb::new(0xf6, 0xc1, 0x77),
    comment: Rgb::new(0x6e, 0x6a, 0x86),
    ty: Rgb::new(0xc4, 0xa7, 0xe7),
    icon_csharp: Rgb::new(0x56, 0x9c, 0xd6),
    icon_markdown: Rgb::new(0x9f, 0xb4, 0xc7),
    icon_folder: Rgb::new(0xb7, 0xb3, 0xc9),
    icon_project: Rgb::new(0xc4, 0xa7, 0xe7),
    icon_file: Rgb::new(0x90, 0x8c, 0xaa),
};

/// The light theme.
pub const LIGHT: Theme = Theme {
    name: "backback light",
    kind: ThemeKind::Light,
    bg: Rgb::new(0xfa, 0xf4, 0xed),
    surface: Rgb::new(0xff, 0xfa, 0xf3),
    overlay: Rgb::new(0xf2, 0xe9, 0xe1),
    border: Rgb::new(0x98, 0x93, 0xa5),
    text: Rgb::new(0x57, 0x52, 0x79),
    text_dim: Rgb::new(0x79, 0x75, 0x93),
    ignored: Rgb::new(0xc7, 0x50, 0x9a),
    chosen: Rgb::new(0xc7, 0x50, 0x9a),
    accent: Rgb::new(0x56, 0x94, 0x9f),
    selection: Rgb::new(0xcf, 0xe1, 0xe5),
    error: Rgb::new(0xb4, 0x63, 0x7a),
    warn: Rgb::new(0xea, 0x9d, 0x34),
    ok: Rgb::new(0x28, 0x7a, 0x4b),
    keyword: Rgb::new(0x28, 0x69, 0x83),
    string: Rgb::new(0xea, 0x9d, 0x34),
    comment: Rgb::new(0x98, 0x93, 0xa5),
    ty: Rgb::new(0x90, 0x7a, 0xa9),
    icon_csharp: Rgb::new(0x28, 0x69, 0x83),
    icon_markdown: Rgb::new(0x6c, 0x84, 0x9c),
    icon_folder: Rgb::new(0x79, 0x75, 0x93),
    icon_project: Rgb::new(0x90, 0x7a, 0xa9),
    icon_file: Rgb::new(0x79, 0x75, 0x93),
};

impl Theme {
    /// Theme for a kind. `None` means "no preference", which is dark.
    #[must_use]
    pub const fn for_kind(kind: Option<ThemeKind>) -> Self {
        match kind {
            Some(ThemeKind::Light) => LIGHT,
            Some(ThemeKind::Dark) | None => DARK,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark() {
        assert_eq!(Theme::for_kind(None).kind, ThemeKind::Dark);
    }

    fn lin(v: u8) -> f64 {
        let s: f64 = f64::from(v) / 255.0;
        if s <= 0.039_28 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }

    fn luminance(c: Rgb) -> f64 {
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }

    fn contrast(a: Rgb, b: Rgb) -> f64 {
        let (x, y): (f64, f64) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// BB-BUG-054: selected text reads (WCAG AA), and the selection shows.
    #[test]
    fn selected_text_reads_and_the_selection_shows_in_both_themes() {
        for theme in [DARK, LIGHT] {
            let reads: f64 = contrast(theme.text, theme.selection);
            assert!(reads >= 4.5, "{}: text on selection {reads:.2}", theme.name);
            let shows: f64 = contrast(theme.selection, theme.bg);
            assert!(shows >= 1.2, "{}: selection on bg {shows:.2}", theme.name);
        }
    }

    #[test]
    fn hex_formats_two_digits_per_channel() {
        assert_eq!(DARK.bg.hex(), "#232136");
        assert_eq!(LIGHT.bg.hex(), "#faf4ed");
    }
}
