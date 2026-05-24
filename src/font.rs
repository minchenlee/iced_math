//! Bundled math font parsing.

use std::fmt::Write;
use std::sync::OnceLock;
use ttf_parser::Face;
pub use ttf_parser::GlyphId;

use crate::FONT_BYTES;

fn face() -> &'static Face<'static> {
    static FACE: OnceLock<Face<'static>> = OnceLock::new();
    FACE.get_or_init(|| Face::parse(FONT_BYTES, 0).expect("bundled math font must parse"))
}

pub fn units_per_em() -> f32 {
    face().units_per_em() as f32
}

// Used only by tests; kept as a small reusable font-capability probe.
#[allow(dead_code)]
pub fn has_math_table() -> bool {
    face().tables().math.is_some()
}

/// Look up the glyph ID for a Unicode codepoint via the font's cmap.
/// Returns `None` if the codepoint is not present in the font.
pub fn glyph_id(ch: char) -> Option<GlyphId> {
    face().glyph_index(ch)
}

/// Map the character pulldown-latex emits for an accent (`^`, `→`, `~`, `‾`/`¯`,
/// …) to the font's **combining** accent glyph (U+03xx / U+20D7). The combining
/// glyphs are the correctly-proportioned over-accent forms (zero advance,
/// designed to overlay), unlike the spacing chars pulldown surfaces. Returns
/// `None` if the character isn't a recognized accent.
pub fn accent_glyph(ch: char) -> Option<GlyphId> {
    let combining = match ch {
        '^' | '\u{0302}' | 'ˆ' => '\u{0302}',  // \hat
        '~' | '\u{0303}' | '˜' => '\u{0303}',  // \tilde
        '‾' | '¯' | '\u{0304}' => '\u{0304}', // \bar (¯ = U+00AF)
        '→' | '\u{20D7}' => '\u{20D7}',        // \vec
        '˙' | '\u{0307}' => '\u{0307}',        // \dot
        '¨' | '\u{0308}' => '\u{0308}',        // \ddot
        'ˇ' | '\u{030C}' => '\u{030C}',        // \check
        '˘' | '\u{0306}' => '\u{0306}',        // \breve
        '´' | '\u{0301}' => '\u{0301}',        // \acute
        '`' | '\u{0300}' => '\u{0300}',        // \grave
        _ => return None,
    };
    face().glyph_index(combining)
}

/// Pixel-space metrics for a glyph at a given font size.
/// All values are in SVG-down y space; `height` is above baseline, `depth` below.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance: f32,
    pub height: f32,
    pub depth: f32,
}

/// Horizontal extent (`x_min`, `x_max`) of a glyph's outline in pixels at a
/// given font size. Useful for glyphs whose advance is zero (combining accents),
/// where the visible width comes from the bounding box, not the advance.
/// Returns `(0.0, 0.0)` if the glyph has no outline.
pub fn glyph_x_bounds(id: GlyphId, font_size: f32) -> (f32, f32) {
    let face = face();
    let scale = font_size / face.units_per_em() as f32;
    match face.glyph_bounding_box(id) {
        Some(b) => (b.x_min as f32 * scale, b.x_max as f32 * scale),
        None => (0.0, 0.0),
    }
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
    GlyphMetrics {
        advance,
        height,
        depth,
    }
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

/// OpenType MATH table constant selector.
///
/// All variants resolve to font design units (via `MathValue.value: i16`) and are
/// scaled to pixels by `math_constant()`, except `RadicalDegreeBottomRaisePercent`
/// which is returned by ttf-parser as `i16` percent and converted to a 0..1 ratio.
// Not all constants are consumed yet; the unused ones back LaTeX features
// scheduled for later tiers (limits, stretch stacks). Kept for completeness.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum MathConstant {
    AxisHeight,
    FractionNumeratorShiftUp,
    FractionDenominatorShiftDown,
    FractionRuleThickness,
    FractionNumDisplayStyleShiftUp,
    FractionDenomDisplayStyleShiftDown,
    FractionNumeratorGapMin,
    FractionNumDisplayStyleGapMin,
    FractionDenominatorGapMin,
    FractionDenomDisplayStyleGapMin,
    SubscriptShiftDown,
    SubscriptTopMax,
    SubscriptBaselineDropMin,
    SuperscriptShiftUp,
    SuperscriptShiftUpCramped,
    SuperscriptBottomMin,
    SuperscriptBaselineDropMax,
    SubSuperscriptGapMin,
    SuperscriptBottomMaxWithSubscript,
    RadicalRuleThickness,
    RadicalVerticalGap,
    RadicalDisplayStyleVerticalGap,
    RadicalKernBeforeDegree,
    RadicalKernAfterDegree,
    /// Returned as a unitless ratio (e.g. 0.6 for 60%), not pixels.
    RadicalDegreeBottomRaisePercent,
    UpperLimitGapMin,
    UpperLimitBaselineRiseMin,
    LowerLimitGapMin,
    LowerLimitBaselineDropMin,
    AccentBaseHeight,
}

/// Read a MATH table constant, scaled to pixels at the given font size.
///
/// Returns `0.0` if the font lacks a MATH table (should not happen with the
/// bundled Latin Modern Math). `RadicalDegreeBottomRaisePercent` returns a
/// 0..1 ratio instead of pixels (font_size is ignored for that variant).
pub fn math_constant(c: MathConstant, font_size: f32) -> f32 {
    let face = face();
    let scale = font_size / face.units_per_em() as f32;
    let Some(math) = face.tables().math else {
        return 0.0;
    };
    let Some(consts) = math.constants else {
        return 0.0;
    };
    use MathConstant::*;
    let value: i16 = match c {
        AxisHeight => consts.axis_height().value,
        FractionNumeratorShiftUp => consts.fraction_numerator_shift_up().value,
        FractionDenominatorShiftDown => consts.fraction_denominator_shift_down().value,
        FractionRuleThickness => consts.fraction_rule_thickness().value,
        FractionNumDisplayStyleShiftUp => consts.fraction_numerator_display_style_shift_up().value,
        FractionDenomDisplayStyleShiftDown => {
            consts.fraction_denominator_display_style_shift_down().value
        }
        FractionNumeratorGapMin => consts.fraction_numerator_gap_min().value,
        FractionNumDisplayStyleGapMin => consts.fraction_num_display_style_gap_min().value,
        FractionDenominatorGapMin => consts.fraction_denominator_gap_min().value,
        FractionDenomDisplayStyleGapMin => consts.fraction_denom_display_style_gap_min().value,
        SubscriptShiftDown => consts.subscript_shift_down().value,
        SubscriptTopMax => consts.subscript_top_max().value,
        SubscriptBaselineDropMin => consts.subscript_baseline_drop_min().value,
        SuperscriptShiftUp => consts.superscript_shift_up().value,
        SuperscriptShiftUpCramped => consts.superscript_shift_up_cramped().value,
        SuperscriptBottomMin => consts.superscript_bottom_min().value,
        SuperscriptBaselineDropMax => consts.superscript_baseline_drop_max().value,
        SubSuperscriptGapMin => consts.sub_superscript_gap_min().value,
        SuperscriptBottomMaxWithSubscript => consts.superscript_bottom_max_with_subscript().value,
        RadicalRuleThickness => consts.radical_rule_thickness().value,
        RadicalVerticalGap => consts.radical_vertical_gap().value,
        RadicalDisplayStyleVerticalGap => consts.radical_display_style_vertical_gap().value,
        RadicalKernBeforeDegree => consts.radical_kern_before_degree().value,
        RadicalKernAfterDegree => consts.radical_kern_after_degree().value,
        RadicalDegreeBottomRaisePercent => {
            return consts.radical_degree_bottom_raise_percent() as f32 / 100.0;
        }
        UpperLimitGapMin => consts.upper_limit_gap_min().value,
        UpperLimitBaselineRiseMin => consts.upper_limit_baseline_rise_min().value,
        LowerLimitGapMin => consts.lower_limit_gap_min().value,
        LowerLimitBaselineDropMin => consts.lower_limit_baseline_drop_min().value,
        AccentBaseHeight => consts.accent_base_height().value,
    };
    value as f32 * scale
}

/// Find the smallest vertical glyph variant whose advance measurement is
/// `>= target_design_units`. If no variant reaches that size, return the
/// **largest available** variant instead — so callers asking for huge
/// delimiters (e.g. around a triple-stacked fraction) still get the biggest
/// glyph the font can provide, rather than silently falling back to the base
/// glyph via `unwrap_or(base)`.
///
/// Returns `None` only when the glyph has no `MathVariants` construction entry
/// (i.e. it's not a stretchy glyph in this font).
///
/// v0.1 ignores `GlyphAssembly` (extensible glyphs built from parts) — that is
/// deferred to v0.2.
pub fn math_variant_vertical(base: GlyphId, target_design_units: f32) -> Option<(GlyphId, f32)> {
    let math = face().tables().math?;
    let variants = math.variants?;
    let construction = variants.vertical_constructions.get(base)?;
    let mut largest: Option<(GlyphId, f32)> = None;
    for v in construction.variants {
        let adv = v.advance_measurement as f32;
        if adv >= target_design_units {
            return Some((v.variant_glyph, adv));
        }
        match largest {
            Some((_, h)) if h >= adv => {}
            _ => largest = Some((v.variant_glyph, adv)),
        }
    }
    largest
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- font_smoke.rs ---
    #[test]
    fn face_loads_and_has_math_table() {
        let upem = units_per_em();
        assert!(upem > 0.0, "units_per_em should be > 0, got {}", upem);
        assert!(has_math_table(), "bundled font must have MATH table");
    }

    // --- font_glyph.rs ---
    #[test]
    fn maps_ascii_letter_to_glyph_id() {
        let id = glyph_id('E').expect("E must exist in bundled math font");
        assert!(id.0 > 0);
    }

    #[test]
    fn maps_greek_alpha() {
        let id = glyph_id('α').expect("α must exist");
        assert!(id.0 > 0);
    }

    #[test]
    fn returns_none_for_unmapped_codepoint() {
        assert!(glyph_id('\u{E000}').is_none());
    }

    // --- font_metrics.rs ---
    #[test]
    fn metrics_for_capital_e() {
        let id = glyph_id('E').unwrap();
        let m = glyph_metrics(id, 16.0);
        assert!(m.advance > 0.0);
        assert!(m.height > 0.0);
        assert!(m.depth >= 0.0);
        assert!(m.height > m.depth);
    }

    #[test]
    fn metrics_scale_linearly_with_size() {
        let id = glyph_id('E').unwrap();
        let m1 = glyph_metrics(id, 10.0);
        let m2 = glyph_metrics(id, 20.0);
        let ratio = m2.advance / m1.advance;
        assert!(
            (ratio - 2.0).abs() < 1e-3,
            "expected 2.0 ratio, got {}",
            ratio
        );
    }

    // --- font_math_table.rs ---
    #[test]
    fn reads_axis_height() {
        let h = math_constant(MathConstant::AxisHeight, 16.0);
        assert!(
            h > 0.0 && h < 16.0,
            "AxisHeight should be small positive px, got {}",
            h
        );
    }

    #[test]
    fn reads_fraction_rule_thickness() {
        let t = math_constant(MathConstant::FractionRuleThickness, 16.0);
        assert!(
            t > 0.0 && t < 2.0,
            "FractionRuleThickness should be ~1px, got {}",
            t
        );
    }

    // --- font_math_variant.rs ---
    #[test]
    fn integral_has_bigger_variant() {
        let id = glyph_id('∫').unwrap();
        let (variant, advance) = math_variant_vertical(id, 1500.0)
            .expect("integral must have a bigger vertical variant");
        assert!(
            variant != id,
            "should return a different glyph for bigger size"
        );
        assert!(advance >= 1500.0);
    }

    #[test]
    fn returns_none_for_atom_without_variants() {
        let id = glyph_id('E').unwrap();
        assert!(math_variant_vertical(id, 50.0).is_none());
    }

    #[test]
    fn returns_largest_when_target_exceeds_all_variants() {
        let id = glyph_id('∫').unwrap();
        let (variant, h) = math_variant_vertical(id, 1e9)
            .expect("should fall back to largest variant, not None");
        assert!(
            variant != id,
            "should return a non-base variant; got base glyph"
        );
        assert!(
            h > 1500.0,
            "largest variant should be substantially bigger than base; got {}",
            h,
        );
    }

    // --- font_outline.rs ---
    #[test]
    fn outlines_capital_e_to_path_string() {
        let id = glyph_id('E').unwrap();
        let path = outline_path(id);
        assert!(
            path.contains('M'),
            "path should contain at least one Move: {}",
            path
        );
        assert!(
            path.contains('L') || path.contains('C') || path.contains('Q'),
            "path should contain at least one line/curve segment: {}",
            path
        );
        assert!(
            path.ends_with('Z') || path.contains("Z "),
            "path should be closed: {}",
            path
        );
    }

    #[test]
    fn outline_uses_design_units_no_scaling() {
        let id = glyph_id('E').unwrap();
        let path = outline_path(id);
        let any_large = path
            .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .filter_map(|s| s.parse::<f32>().ok())
            .any(|n| n.abs() > 50.0);
        assert!(any_large, "expected design-unit magnitudes, got: {}", path);
    }
}
