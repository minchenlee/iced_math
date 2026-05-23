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
    Glyph {
        glyph_id: ttf_parser::GlyphId,
        font_size: f32,
    },
    HBox(Vec<Child>),
    VBox(Vec<Child>),
    Rule {
        thickness: f32,
    },
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
        Node::Atom {
            glyph, font_size, ..
        } => {
            // `font_size` is the BASE size (e.g. 16.0); apply style scaling here
            // so script-position atoms render at script-scale glyph metrics.
            let actual = style.font_size(*font_size);
            let m = font::glyph_metrics(*glyph, actual);
            Box {
                width: m.advance,
                height: m.height,
                depth: m.depth,
                kind: BoxKind::Glyph {
                    glyph_id: *glyph,
                    font_size: actual,
                },
            }
        }
        Node::Row(items) => layout_row(items, style),
        Node::Frac { num, den } => layout_frac(num, den, style),
        Node::Subsup { base, sub, sup } => {
            layout_subsup(base, sub.as_deref(), sup.as_deref(), style)
        }
        Node::Radical { degree, body } => layout_radical(degree.as_deref(), body, style),
        Node::Fenced { open, close, body } => layout_fenced(*open, *close, body, style),
        Node::Op { .. } => layout_op(node, style),
        Node::Error(_) => Box {
            width: 0.0,
            height: 0.0,
            depth: 0.0,
            kind: BoxKind::Empty,
        },
        _ => Box {
            width: 0.0,
            height: 0.0,
            depth: 0.0,
            kind: BoxKind::Empty,
        },
    }
}

/// Layout `base` with optional sub/superscripts. If `base` is a big operator
/// flagged with `limits: true`, stack scripts vertically (see [`layout_with_limits`]);
/// otherwise place scripts to the right of `base`, sized down by [`Style::sub`].
fn layout_subsup(base: &Node, sub: Option<&Node>, sup: Option<&Node>, style: Style) -> Box {
    use crate::font::{math_constant, MathConstant};

    // Limits branch: stack scripts above/below big-op base.
    if let Node::Op { limits: true, .. } = base {
        return layout_with_limits(base, sub, sup, style);
    }

    let b = layout(base, style);
    let script_style = style.sub();
    let s_sup = sup.map(|n| layout(n, script_style));
    let s_sub = sub.map(|n| layout(n, script_style));
    let base_size = style.font_size(approx_base_font_size_from_node(base));

    let sup_shift = math_constant(MathConstant::SuperscriptShiftUp, base_size);
    let sub_shift = math_constant(MathConstant::SubscriptShiftDown, base_size);

    // sup baseline sits at (base baseline - sup_shift); sub baseline at (base baseline + sub_shift).
    let sup_ascent = s_sup.as_ref().map(|s| sup_shift + s.height).unwrap_or(0.0);
    let sub_descent = s_sub.as_ref().map(|s| sub_shift + s.depth).unwrap_or(0.0);

    let parent_height = b.height.max(sup_ascent);
    let parent_depth = b.depth.max(sub_descent);

    let scripts_x = b.width;
    let mut max_w = b.width;

    let b_width = b.width;
    let b_height = b.height;
    let mut children = Vec::new();
    // Base: baseline aligns with parent baseline.
    children.push(Child {
        offset: Point {
            x: 0.0,
            y: parent_height - b_height,
        },
        child: b,
    });

    if let Some(sup_box) = s_sup {
        let sup_baseline_y = parent_height - sup_shift;
        let sup_top = sup_baseline_y - sup_box.height;
        max_w = max_w.max(scripts_x + sup_box.width);
        children.push(Child {
            offset: Point {
                x: scripts_x,
                y: sup_top,
            },
            child: sup_box,
        });
    }
    if let Some(sub_box) = s_sub {
        let sub_baseline_y = parent_height + sub_shift;
        let sub_top = sub_baseline_y - sub_box.height;
        max_w = max_w.max(scripts_x + sub_box.width);
        children.push(Child {
            offset: Point {
                x: scripts_x,
                y: sub_top,
            },
            child: sub_box,
        });
    }

    let _ = b_width; // base width is captured in scripts_x already
    Box {
        width: max_w,
        height: parent_height,
        depth: parent_depth,
        kind: BoxKind::HBox(children),
    }
}

/// Layout `\sqrt{body}` or `\sqrt[degree]{body}`. Uses the surd glyph (U+221A)
/// scaled up via MATH vertical variants to span the body's height; rule
/// (vinculum) extends across the body.
fn layout_radical(degree: Option<&Node>, body: &Node, style: Style) -> Box {
    use crate::font::{self, math_constant, MathConstant};

    let body_box = layout(body, style);
    let base_size = style.font_size(approx_base_font_size_from_node(body));

    let rule_thickness = math_constant(MathConstant::RadicalRuleThickness, base_size).max(0.5);
    let gap = if style.is_display() {
        math_constant(MathConstant::RadicalDisplayStyleVerticalGap, base_size)
    } else {
        math_constant(MathConstant::RadicalVerticalGap, base_size)
    };

    let surd_base = font::glyph_id('√').expect("√ must exist in math font");
    let needed_design_units = (body_box.height + body_box.depth + gap + rule_thickness) / base_size
        * font::units_per_em();
    let surd_id = font::math_variant_vertical(surd_base, needed_design_units)
        .map(|(g, _)| g)
        .unwrap_or(surd_base);
    let surd_m = font::glyph_metrics(surd_id, base_size);

    let surd_box = Box {
        width: surd_m.advance,
        height: surd_m.height,
        depth: surd_m.depth,
        kind: BoxKind::Glyph {
            glyph_id: surd_id,
            font_size: base_size,
        },
    };

    let surd_w = surd_box.width;
    let surd_h = surd_box.height + surd_box.depth;
    let body_w = body_box.width;
    let body_h = body_box.height;
    let body_d = body_box.depth;

    // Total vertical extent above baseline: at least rule + gap + body.height.
    let inner_ascent = rule_thickness + gap + body_h;
    let parent_height = surd_box.height.max(inner_ascent);
    let parent_depth = body_d.max(surd_box.depth);

    let rule_y = parent_height - body_h - gap - rule_thickness;
    let body_y = parent_height - body_h;
    let surd_y = parent_height - surd_box.height;

    let mut children = vec![
        Child {
            offset: Point { x: 0.0, y: surd_y },
            child: surd_box,
        },
        Child {
            offset: Point {
                x: surd_w,
                y: rule_y,
            },
            child: Box {
                width: body_w,
                height: rule_thickness,
                depth: 0.0,
                kind: BoxKind::Rule {
                    thickness: rule_thickness,
                },
            },
        },
        Child {
            offset: Point {
                x: surd_w,
                y: body_y,
            },
            child: body_box,
        },
    ];

    // Vertical connector: when the surd glyph's bounding box top sits below
    // the overline (parent_height > surd.height ⇒ surd_y > rule_y), there is
    // a visible gap between the surd's hook/peak and the start of the
    // vinculum. Bridge it with a thin vertical rule aligned to the right
    // edge of the surd, spanning from `rule_y` (top of overline) down past
    // `surd_y` (top of surd bbox) with a small overlap to avoid hairline
    // seams. Mirrors KaTeX's approach of extending the surd stem to meet
    // the vinculum.
    if surd_y > rule_y {
        let overlap = rule_thickness;
        let connector_h = (surd_y - rule_y) + overlap;
        let connector_x = (surd_w - rule_thickness).max(0.0);
        children.push(Child {
            offset: Point {
                x: connector_x,
                y: rule_y,
            },
            child: Box {
                width: rule_thickness,
                height: connector_h,
                depth: 0.0,
                kind: BoxKind::Rule {
                    thickness: connector_h,
                },
            },
        });
    }

    // Degree (n in \sqrt[n]{x}) — placed above-left of the surd's lower point.
    // We shift every existing child rightward by the degree's width + the
    // RadicalKernAfterDegree kern, then place the degree at x=0. Without this
    // shift, a wide degree (e.g. `\sqrt[12345]{x}`) overlaps the surd and
    // extends past the parent box's reported width, causing SVG clipping.
    let mut total_width = surd_w + body_w;
    if let Some(deg) = degree {
        let deg_box = layout(deg, Style::ScriptScript);
        let kern_after = math_constant(MathConstant::RadicalKernAfterDegree, base_size).max(0.0);
        let raise_pct = math_constant(MathConstant::RadicalDegreeBottomRaisePercent, base_size);
        let deg_h = deg_box.height;
        let deg_y = (parent_height - (surd_h * raise_pct) - deg_h).max(0.0);
        let shift = deg_box.width + kern_after;
        for ch in &mut children {
            ch.offset.x += shift;
        }
        let deg_w = deg_box.width;
        children.insert(
            0,
            Child {
                offset: Point { x: 0.0, y: deg_y },
                child: deg_box,
            },
        );
        total_width = shift + surd_w + body_w;
        let _ = deg_w;
    }

    Box {
        width: total_width,
        height: parent_height,
        depth: parent_depth,
        kind: BoxKind::HBox(children),
    }
}

/// Layout `\left<open> body \right<close>`. The open/close glyphs are sized
/// via MATH vertical variants to span the body's ink height. `GlyphId(0)` is
/// the invisible-delimiter sentinel (LaTeX `.`) and is rendered as a zero-width
/// empty box.
fn layout_fenced(
    open: ttf_parser::GlyphId,
    close: ttf_parser::GlyphId,
    body: &Node,
    style: Style,
) -> Box {
    use crate::font;

    let body_box = layout(body, style);
    let base_size = style.font_size(approx_base_font_size_from_node(body));
    let body_total_du = (body_box.height + body_box.depth) / base_size * font::units_per_em();

    fn pick_variant(base: ttf_parser::GlyphId, target_du: f32) -> ttf_parser::GlyphId {
        if base.0 == 0 {
            return base;
        }
        font::math_variant_vertical(base, target_du)
            .map(|(g, _)| g)
            .unwrap_or(base)
    }
    let open_id = pick_variant(open, body_total_du);
    let close_id = pick_variant(close, body_total_du);

    fn glyph_box(id: ttf_parser::GlyphId, size: f32) -> Box {
        if id.0 == 0 {
            return Box {
                width: 0.0,
                height: 0.0,
                depth: 0.0,
                kind: BoxKind::Empty,
            };
        }
        let m = font::glyph_metrics(id, size);
        Box {
            width: m.advance,
            height: m.height,
            depth: m.depth,
            kind: BoxKind::Glyph {
                glyph_id: id,
                font_size: size,
            },
        }
    }
    let open_box = glyph_box(open_id, base_size);
    let close_box = glyph_box(close_id, base_size);

    let parent_height = body_box.height.max(open_box.height).max(close_box.height);
    let parent_depth = body_box.depth.max(open_box.depth).max(close_box.depth);

    let mut children = Vec::new();
    let mut x = 0.0;
    for b in [open_box, body_box, close_box] {
        let w = b.width;
        let h = b.height;
        children.push(Child {
            offset: Point {
                x,
                y: parent_height - h,
            },
            child: b,
        });
        x += w;
    }

    Box {
        width: x,
        height: parent_height,
        depth: parent_depth,
        kind: BoxKind::HBox(children),
    }
}

/// Layout a bare big-op (e.g. `\sum`, `\int`) as a single glyph box.
fn layout_op(node: &Node, style: Style) -> Box {
    let Node::Op {
        glyph, font_size, ..
    } = node
    else {
        return Box {
            width: 0.0,
            height: 0.0,
            depth: 0.0,
            kind: BoxKind::Empty,
        };
    };
    // `font_size` is the BASE size; apply style scaling for correct glyph metrics.
    let actual = style.font_size(*font_size);
    let m = crate::font::glyph_metrics(*glyph, actual);
    Box {
        width: m.advance,
        height: m.height,
        depth: m.depth,
        kind: BoxKind::Glyph {
            glyph_id: *glyph,
            font_size: actual,
        },
    }
}

/// Stack sub/sup limits vertically above/below a big-op base (display mode).
fn layout_with_limits(base: &Node, sub: Option<&Node>, sup: Option<&Node>, style: Style) -> Box {
    use crate::font::{math_constant, MathConstant};

    let b = layout_op(base, style);
    let script_style = style.sub();
    let s_sup = sup.map(|n| layout(n, script_style));
    let s_sub = sub.map(|n| layout(n, script_style));

    let base_size = approx_font_size_from_box(&b);
    let upper_gap = math_constant(MathConstant::UpperLimitGapMin, base_size);
    let lower_gap = math_constant(MathConstant::LowerLimitGapMin, base_size);

    let sup_total = s_sup.as_ref().map(|s| s.height + s.depth).unwrap_or(0.0);
    let sub_total = s_sub.as_ref().map(|s| s.height + s.depth).unwrap_or(0.0);

    let b_w = b.width;
    let b_h = b.height;
    let b_d = b.depth;

    let width = b_w
        .max(s_sup.as_ref().map(|s| s.width).unwrap_or(0.0))
        .max(s_sub.as_ref().map(|s| s.width).unwrap_or(0.0));

    let parent_height = if s_sup.is_some() {
        b_h + upper_gap + sup_total
    } else {
        b_h
    };
    let parent_depth = if s_sub.is_some() {
        b_d + lower_gap + sub_total
    } else {
        b_d
    };

    let base_top = parent_height - b_h;
    let base_x = (width - b_w) / 2.0;
    let mut children = vec![Child {
        offset: Point {
            x: base_x,
            y: base_top,
        },
        child: b,
    }];

    if let Some(sup_box) = s_sup {
        let sup_w = sup_box.width;
        let sup_total_h = sup_box.height + sup_box.depth;
        let sup_top = base_top - upper_gap - sup_total_h;
        let sup_x = (width - sup_w) / 2.0;
        children.push(Child {
            offset: Point {
                x: sup_x,
                y: sup_top.max(0.0),
            },
            child: sup_box,
        });
    }
    if let Some(sub_box) = s_sub {
        let sub_w = sub_box.width;
        let sub_top = parent_height + b_d + lower_gap;
        let sub_x = (width - sub_w) / 2.0;
        children.push(Child {
            offset: Point {
                x: sub_x,
                y: sub_top,
            },
            child: sub_box,
        });
    }

    Box {
        width,
        height: parent_height,
        depth: parent_depth,
        kind: BoxKind::VBox(children),
    }
}

fn layout_row(items: &[Node], style: Style) -> Box {
    use crate::spacing;

    // Pass 1: lay out children + horizontal cursor, accumulating max height/depth.
    struct Placed {
        x: f32,
        b: Box,
    }
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
            cursor += sp.to_px(style.font_size(approx_font_size(node)));
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
            Child {
                offset: Point { x: p.x, y },
                child: p.b,
            }
        })
        .collect();

    Box {
        width: cursor,
        height,
        depth,
        kind: BoxKind::HBox(children),
    }
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

    // Base size for MATH constants is the fraction's outer style applied to
    // its content's base font_size — NOT the children's actual (sub-styled)
    // rendered size. Otherwise rule thickness, shifts etc. shrink along with
    // the inner glyphs and the fraction looks anemic.
    let base_content =
        approx_base_font_size_from_node(num).max(approx_base_font_size_from_node(den));
    let base = style.font_size(base_content);
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
        Child {
            offset: Point {
                x: num_x,
                y: num_top,
            },
            child: n,
        },
        Child {
            offset: Point {
                x: 0.0,
                y: rule_top,
            },
            child: Box {
                width,
                height: rule_thickness,
                depth: 0.0,
                kind: BoxKind::Rule {
                    thickness: rule_thickness,
                },
            },
        },
        Child {
            offset: Point {
                x: den_x,
                y: den_top,
            },
            child: d,
        },
    ];

    Box {
        width,
        height: parent_height,
        depth: parent_depth,
        kind: BoxKind::VBox(children),
    }
}

/// Walks a Box tree to find a representative leaf-glyph font_size.
/// Returned values are **actual rendered** sizes (style scaling already applied).
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

/// Walks a Node tree to find a representative **base** font_size (i.e. the
/// unscaled size stored on Atom/Op nodes). Use this when computing MATH
/// constants for a layout that operates on a node and needs the base size of
/// its content — independent of whatever sub-style its children may use
/// internally (e.g. fraction internals stepping down to Script style).
fn approx_base_font_size_from_node(n: &Node) -> f32 {
    match n {
        Node::Atom { font_size, .. } | Node::Op { font_size, .. } => *font_size,
        Node::Row(items) => items
            .first()
            .map(approx_base_font_size_from_node)
            .unwrap_or(16.0),
        Node::Frac { num, .. } => approx_base_font_size_from_node(num),
        Node::Subsup { base, .. } => approx_base_font_size_from_node(base),
        Node::Radical { body, .. } => approx_base_font_size_from_node(body),
        Node::Fenced { body, .. } => approx_base_font_size_from_node(body),
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
