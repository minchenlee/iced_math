//! Bundled math font parsing.

use std::fmt::Write;
use std::sync::OnceLock;
use ttf_parser::Face;
pub use ttf_parser::GlyphId;

use crate::FONT_BYTES;

fn face() -> &'static Face<'static> {
    static FACE: OnceLock<Face<'static>> = OnceLock::new();
    FACE.get_or_init(|| {
        Face::parse(FONT_BYTES, 0).expect("bundled math font must parse")
    })
}

pub fn units_per_em() -> f32 {
    face().units_per_em() as f32
}

pub fn has_math_table() -> bool {
    face().tables().math.is_some()
}

/// Look up the glyph ID for a Unicode codepoint via the font's cmap.
/// Returns `None` if the codepoint is not present in the font.
pub fn glyph_id(ch: char) -> Option<GlyphId> {
    face().glyph_index(ch)
}

/// Pixel-space metrics for a glyph at a given font size.
/// All values are in SVG-down y space; `height` is above baseline, `depth` below.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance: f32,
    pub height: f32,
    pub depth: f32,
}

pub fn glyph_metrics(id: GlyphId, font_size: f32) -> GlyphMetrics {
    let face = face();
    let upem = face.units_per_em() as f32;
    let scale = font_size / upem;

    let advance = face.glyph_hor_advance(id).unwrap_or(0) as f32 * scale;
    let bbox = face.glyph_bounding_box(id);
    let (height, depth) = match bbox {
        Some(b) => (b.y_max as f32 * scale, (-(b.y_min as f32)) * scale),
        None => (0.0, 0.0),
    };
    GlyphMetrics { advance, height, depth }
}

struct PathBuilder(String);

impl ttf_parser::OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.0, "M{} {} ", x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.0, "L{} {} ", x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let _ = write!(self.0, "Q{} {} {} {} ", x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let _ = write!(self.0, "C{} {} {} {} {} {} ", x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.0.push_str("Z ");
    }
}

/// Emit the glyph's outline as an SVG path data string in font design units (y-up).
/// Caller MUST apply `matrix(s 0 0 -s ox oy)` transform where `s = font_size / units_per_em`
/// to convert to SVG (y-down) pixel space.
/// Returns empty string for blank glyphs.
pub fn outline_path(id: GlyphId) -> String {
    let mut b = PathBuilder(String::new());
    let _ = face().outline_glyph(id, &mut b);
    b.0
}
