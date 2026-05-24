//! Public color type for math rendering. Kept iced-free so the low-level
//! `to_svg` path has no GUI dependency.

/// An sRGB color (no alpha). Used for the fill of rendered math glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Opaque black — the default math fill.
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };

    /// Construct from 8-bit sRGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }

    /// `#rrggbb` hex string for SVG `fill`.
    pub(crate) fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_formats_lowercase_padded() {
        assert_eq!(Color::rgb(0xab, 0x01, 0xff).hex(), "#ab01ff");
        assert_eq!(Color::BLACK.hex(), "#000000");
    }

    #[test]
    fn default_is_black() {
        assert_eq!(Color::default(), Color::BLACK);
    }
}
