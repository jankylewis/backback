//! One font and one size for every surface (BBEOD08), and zoom (BBEOD14).
//! The size a surface uses is `base + zoom`; Cmd+0 clears only the zoom.
//! Saved as a small versioned text file by the desktop app.

/// The default family: the same font everywhere.
pub const DEFAULT_FAMILY: &str = "Menlo";
/// The default size in points.
pub const DEFAULT_SIZE: f32 = 13.0;
/// Smallest and largest size zoom can reach.
pub const MIN_SIZE: f32 = 8.0;
/// See [`MIN_SIZE`].
pub const MAX_SIZE: f32 = 32.0;
const HEADER: &str = "backback settings v1";

/// The font settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Typography {
    /// Font family, e.g. `Menlo`.
    pub family: String,
    /// Size chosen in Settings, in points.
    pub base: f32,
    /// Zoom steps (one point each), from Cmd+= and Cmd+-.
    pub zoom: i8,
    /// The tool window (Explorer / Tests) docks on the right, not the left.
    pub panel_right: bool,
}

/// A settings file this version cannot read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SettingsError {
    /// The first line is not this version's header.
    #[error("settings: unknown format {0:?}")]
    UnknownFormat(String),
    /// A line is not `key=value` with a known key and a valid value.
    #[error("settings: bad line {0:?}")]
    BadLine(String),
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            family: DEFAULT_FAMILY.to_owned(),
            base: DEFAULT_SIZE,
            zoom: 0,
            panel_right: false,
        }
    }
}

impl Typography {
    /// The size every surface draws text at.
    #[must_use]
    pub fn size(&self) -> f32 {
        (self.base + f32::from(self.zoom)).clamp(MIN_SIZE, MAX_SIZE)
    }

    /// A text row's height: 22 px at 13 pt, scaled with the size.
    #[must_use]
    pub fn line_height(&self) -> f32 {
        (self.size() * 1.7).round()
    }

    /// The editor's line-number gutter: 50 px at 13 pt, scaled with the size,
    /// so numbers never run into the code when zoomed.
    #[must_use]
    pub fn gutter_width(&self) -> f32 {
        (self.size() * 50.0 / DEFAULT_SIZE).round()
    }

    /// Where the editor's text starts in the window: after the tool window
    /// (`panel_width` + its 6 px edge) when it docks left, then the gutter.
    #[must_use]
    pub fn editor_x(&self, panel_width: f32) -> f32 {
        let left: f32 = if self.panel_right {
            0.0
        } else {
            panel_width + 6.0
        };
        left + self.gutter_width()
    }

    /// The Settings size, one point up or down, within bounds.
    pub fn step_base(&mut self, up: bool) {
        let next: f32 = if up { self.base + 1.0 } else { self.base - 1.0 };
        self.base = next.clamp(MIN_SIZE, MAX_SIZE);
    }

    /// One step bigger, up to [`MAX_SIZE`].
    pub fn zoom_in(&mut self) {
        if self.size() < MAX_SIZE {
            self.zoom = self.zoom.saturating_add(1);
        }
    }

    /// One step smaller, down to [`MIN_SIZE`].
    pub fn zoom_out(&mut self) {
        if self.size() > MIN_SIZE {
            self.zoom = self.zoom.saturating_sub(1);
        }
    }

    /// Back to the size chosen in Settings.
    pub fn zoom_reset(&mut self) {
        self.zoom = 0;
    }

    /// The settings file text.
    #[must_use]
    pub fn encode(&self) -> String {
        format!(
            "{HEADER}\nfont={}\nsize={}\nzoom={}\nside={}\n",
            self.family,
            self.base,
            self.zoom,
            if self.panel_right { "right" } else { "left" }
        )
    }

    /// Read a settings file; missing keys keep their default.
    ///
    /// # Errors
    /// An unknown header, an unknown key or a bad value.
    pub fn decode(text: &str) -> Result<Self, SettingsError> {
        let mut lines: std::str::Lines<'_> = text.lines();
        let header: &str = lines.next().unwrap_or_default();
        if header != HEADER {
            return Err(SettingsError::UnknownFormat(header.to_owned()));
        }
        let mut out: Self = Self::default();
        for line in lines.filter(|line: &&str| !line.trim().is_empty()) {
            let (key, value): (&str, &str) = line.split_once('=').ok_or_else(|| bad(line))?;
            match key {
                "font" if !value.is_empty() => value.clone_into(&mut out.family),
                "size" => {
                    out.base = value
                        .parse::<f32>()
                        .ok()
                        .filter(|size: &f32| (MIN_SIZE..=MAX_SIZE).contains(size))
                        .ok_or_else(|| bad(line))?;
                }
                "zoom" => {
                    out.zoom = value
                        .parse::<i8>()
                        .map_err(|_: std::num::ParseIntError| bad(line))?;
                }
                "side" if value == "left" || value == "right" => out.panel_right = value == "right",
                _ => return Err(bad(line)),
            }
        }
        Ok(out)
    }
}

fn bad(line: &str) -> SettingsError {
    SettingsError::BadLine(line.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_menlo_13_on_a_22_px_row() {
        let t: Typography = Typography::default();
        assert_eq!(
            (t.family.as_str(), t.size(), t.line_height()),
            ("Menlo", 13.0, 22.0)
        );
        assert_eq!(t.gutter_width(), 50.0);
        let big: Typography = Typography {
            zoom: 13,
            ..Typography::default()
        };
        assert_eq!(big.gutter_width(), 100.0, "the gutter grows with the text");
    }

    #[test]
    fn zoom_moves_one_point_stays_in_bounds_and_reset_keeps_the_base() {
        let mut t: Typography = Typography {
            base: 31.0,
            ..Typography::default()
        };
        t.zoom_in();
        t.zoom_in();
        t.zoom_in();
        assert_eq!(t.size(), MAX_SIZE);
        t.zoom_reset();
        assert_eq!(t.size(), 31.0);
        for _ in 0..40 {
            t.zoom_out();
        }
        assert_eq!(t.size(), MIN_SIZE);
    }

    #[test]
    fn settings_round_trip_and_unknown_input_is_refused() {
        let t: Typography = Typography {
            family: "SF Mono".to_owned(),
            base: 15.0,
            zoom: -2,
            panel_right: true,
        };
        assert_eq!(Typography::decode(&t.encode()), Ok(t));
        assert!(matches!(
            Typography::decode("backback settings v9\n"),
            Err(SettingsError::UnknownFormat(_))
        ));
        assert!(matches!(
            Typography::decode("backback settings v1\ncolor=red\n"),
            Err(SettingsError::BadLine(_))
        ));
        assert!(matches!(
            Typography::decode("backback settings v1\nsize=200\n"),
            Err(SettingsError::BadLine(_))
        ));
    }
}
