//! IR → positioned Box tree.
//!
//! # Coordinate convention
//!
//! **SVG y-down. `(0, 0)` is the top-left of each `Box`'s bounding rectangle.**
//!
//! For a `Box { width, height, depth, .. }`:
//! - The box occupies the rectangle `[0, width] × [0, height + depth]`.
//! - The **baseline** sits at `y = height` (i.e. `height` units below the top).
//!   Glyph ink above the baseline lives in `[0, height]`; descenders live in
//!   `[height, height + depth]`.
//! - `Child.offset` is the **top-left** of the child relative to the parent's
//!   top-left, also in y-down coordinates.
//!
//! ## Baseline alignment in an `HBox` (row)
//!
//! All children share the parent's baseline. The parent's `height` is
//! `max(child.height)` and parent's `depth` is `max(child.depth)`. To place a
//! child whose own baseline-to-top distance is `child.height` so that its
//! baseline coincides with the parent's baseline at `y = parent.height`, set:
//!
//! ```text
//! child.offset.y = parent.height - child.height
//! ```
//!
//! ## Fractions — baseline coincides with the math axis
//!
//! For a `Frac` box, the parent's baseline (the line at `y = parent.height`)
//! is the **math axis line**, not a text baseline. The numerator sits above
//! the axis by `FractionNumeratorShiftUp`; the denominator's baseline sits
//! below the axis by `FractionDenominatorShiftDown`. The fraction rule is
//! centered on the axis. This matches KaTeX/MathML semantics so fractions
//! align vertically with the math axis when composed in surrounding rows.
//! Row-vs-axis alignment in mixed contexts is refined in later tasks.

use crate::font;
use crate::ir::{Node, Style};

#[derive(Debug, Clone)]
pub struct Box {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub kind: BoxKind,
}

#[derive(Debug, Clone)]
pub enum BoxKind {
    Glyph { glyph_id: ttf_parser::GlyphId, font_size: f32 },
    HBox(Vec<Child>),
    VBox(Vec<Child>),
    Rule { thickness: f32 },
    Empty,
}

#[derive(Debug, Clone)]
pub struct Child {
    pub offset: Point,
    pub child: Box,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub fn layout(node: &Node, style: Style) -> Box {
    match node {
        Node::Atom { glyph, font_size, .. } => {
            let m = font::glyph_metrics(*glyph, *font_size);
            Box {
                width: m.advance,
                height: m.height,
                depth: m.depth,
                kind: BoxKind::Glyph { glyph_id: *glyph, font_size: *font_size },
            }
        }
        Node::Row(items) => layout_row(items, style),
        Node::Frac { num, den } => layout_frac(num, den, style),
        Node::Error(_) => Box { width: 0.0, height: 0.0, depth: 0.0, kind: BoxKind::Empty },
        _ => Box { width: 0.0, height: 0.0, depth: 0.0, kind: BoxKind::Empty },
    }
}

fn layout_row(items: &[Node], style: Style) -> Box {
    use crate::spacing;

    // Pass 1: lay out children + horizontal cursor, accumulating max height/depth.
    struct Placed { x: f32, b: Box }
    let mut placed: Vec<Placed> = Vec::new();
    let mut cursor = 0.0_f32;
    let mut height: f32 = 0.0;
    let mut depth: f32 = 0.0;

    let display_or_text = matches!(style, Style::Display | Style::Text);

    let mut prev_class: Option<crate::ir::AtomClass> = None;
    for node in items {
        let cur_class = atom_class(node);
        if let (Some(pc), Some(cc)) = (prev_class, cur_class) {
            let sp = spacing::between(pc, cc, display_or_text);
            cursor += sp.to_px(approx_font_size(node));
        }
        let b = layout(node, style);
        height = height.max(b.height);
        depth = depth.max(b.depth);
        let width = b.width;
        placed.push(Placed { x: cursor, b });
        cursor += width;
        prev_class = cur_class;
    }

    // Pass 2: baseline-align — each child sits with its baseline on the parent's
    // baseline at y = height, i.e. top at y = height - child.height (y-down).
    let children = placed
        .into_iter()
        .map(|p| {
            let y = height - p.b.height;
            Child { offset: Point { x: p.x, y }, child: p.b }
        })
        .collect();

    Box { width: cursor, height, depth, kind: BoxKind::HBox(children) }
}

/// Layout `\frac{num}{den}`. The resulting box's **baseline coincides with
/// the math axis line**; the rule is centered on that axis. See the module
/// doc-comment for the y-down convention.
fn layout_frac(num: &Node, den: &Node, style: Style) -> Box {
    use crate::font::{math_constant, MathConstant};

    // Sub-styles: display fracs keep num/den at text style; otherwise step down.
    let inner_style = match style {
        Style::Display => Style::Text,
        _ => style.sub(),
    };
    let n = layout(num, inner_style);
    let d = layout(den, inner_style);

    // Use the larger of the two children's font size as the base for MATH
    // constant scaling (a reasonable approximation in v0.1).
    let base = approx_font_size_from_box(&n).max(approx_font_size_from_box(&d));
    let rule_thickness = math_constant(MathConstant::FractionRuleThickness, base).max(0.5);

    let (shift_up, shift_down) = if style.is_display() {
        (
            math_constant(MathConstant::FractionNumDisplayStyleShiftUp, base),
            math_constant(MathConstant::FractionDenomDisplayStyleShiftDown, base),
        )
    } else {
        (
            math_constant(MathConstant::FractionNumeratorShiftUp, base),
            math_constant(MathConstant::FractionDenominatorShiftDown, base),
        )
    };

    // Parent baseline (y = parent.height) IS the math axis line.
    //   - num baseline sits at axis - shift_up (above the axis)
    //   - den baseline sits at axis + shift_down (below the axis)
    //
    // parent.height = distance from axis up to top of numerator
    //               = shift_up + n.height
    // parent.depth  = distance from axis down to bottom of denominator
    //               = shift_down + d.depth
    //
    // (Numerator's depth and denominator's height live in the band between
    // axis and the respective glyph baseline; MATH constants are set wide
    // enough by LMM that ascenders/descenders don't cross the rule.)
    let parent_height = shift_up + n.height;
    let parent_depth = shift_down + d.depth;
    let width = n.width.max(d.width);

    let num_x = (width - n.width) / 2.0;
    let den_x = (width - d.width) / 2.0;

    let axis_y = parent_height;
    let num_baseline_y = axis_y - shift_up;
    let num_top = num_baseline_y - n.height;
    let den_baseline_y = axis_y + shift_down;
    let den_top = den_baseline_y - d.height;
    let rule_top = axis_y - rule_thickness / 2.0;

    let children = vec![
        Child { offset: Point { x: num_x, y: num_top }, child: n },
        Child {
            offset: Point { x: 0.0, y: rule_top },
            child: Box {
                width,
                height: rule_thickness,
                depth: 0.0,
                kind: BoxKind::Rule { thickness: rule_thickness },
            },
        },
        Child { offset: Point { x: den_x, y: den_top }, child: d },
    ];

    Box {
        width,
        height: parent_height,
        depth: parent_depth,
        kind: BoxKind::VBox(children),
    }
}

fn approx_font_size_from_box(b: &Box) -> f32 {
    match &b.kind {
        BoxKind::Glyph { font_size, .. } => *font_size,
        BoxKind::HBox(c) | BoxKind::VBox(c) => c
            .first()
            .map(|x| approx_font_size_from_box(&x.child))
            .unwrap_or(16.0),
        _ => 16.0,
    }
}

fn atom_class(node: &Node) -> Option<crate::ir::AtomClass> {
    match node {
        Node::Atom { class, .. } => Some(*class),
        Node::Op { .. } => Some(crate::ir::AtomClass::Op),
        Node::Fenced { .. } => Some(crate::ir::AtomClass::Inner),
        Node::Frac { .. } | Node::Radical { .. } | Node::Subsup { .. } | Node::Row(_) => {
            Some(crate::ir::AtomClass::Ord)
        }
        _ => None,
    }
}

fn approx_font_size(node: &Node) -> f32 {
    match node {
        Node::Atom { font_size, .. } | Node::Op { font_size, .. } => *font_size,
        _ => 16.0,
    }
}
