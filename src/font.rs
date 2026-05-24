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
