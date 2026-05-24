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
        Node::Frac { num, den, bar } => layout_frac(num, den, *bar, style),
        Node::Subsup { base, sub, sup } => {
            layout_subsup(base, sub.as_deref(), sup.as_deref(), style)
        }
        Node::Radical { degree, body } => layout_radical(degree.as_deref(), body, style),
        Node::Fenced { open, close, body } => layout_fenced(*open, *close, body, style),
        Node::Op { .. } => layout_op(node, style),
        Node::OpName { body, .. } => layout(body, style),
        Node::Accent { accent, body } => layout_accent(*accent, body, style),
        Node::Matrix { rows, col_aligns } => layout_matrix(rows, col_aligns, style),
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

    // Limits branch: stack scripts above/below big-op or named-op base.
    if matches!(
        base,
        Node::Op { limits: true, .. } | Node::OpName { limits: true, .. }
    ) {
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
    // Total glyph extent (ascent + descent). The surd's ink TOP edge is its
    // box top (y = box_top); the long diagonal descends to box_top + surd_h.
    let surd_h = surd_box.height + surd_box.depth;
    let body_w = body_box.width;
    let body_h = body_box.height;
    let body_d = body_box.depth;

    // KaTeX/TeX geometry (y-down):
    //
    //   - The vinculum (overline rule) starts at the surd's advance width
    //     (x = surd_w) and runs across the radicand.
    //   - The surd glyph is positioned so its TOP edge (the peak / hook, the
    //     glyph's y_max) coincides exactly with the vinculum's TOP. In Latin
    //     Modern Math the surd's top-right ink reaches the advance width, so
    //     aligning glyph-top to rule-top fuses the hook, stem and vinculum
    //     into one continuous stroke — no separate connector rect needed.
    //   - The radicand sits to the right of the surd, its top `gap` below the
    //     vinculum's bottom. The surd must descend far enough to cover the
    //     radicand's full height+depth below the vinculum.
    //
    // Lay everything out relative to the vinculum top = `rule_y`, then derive
    // the parent baseline from the radicand (so the radical composes in rows).
    //
    // Required surd extent below the vinculum top: rule + gap + body(h+d).
    let inner_below_rule = rule_thickness + gap + body_h + body_d;
    // If the chosen surd is shorter than needed, the radicand would poke below
    // the surd's diagonal; clamp the effective extent so the box stays
    // consistent (the variant picker already aims for >= this, but the base
    // glyph may fall short for very tall radicands).
    let surd_below_rule = surd_h.max(inner_below_rule);

    // Surd box top sits at the vinculum top. Place the vinculum so the surd's
    // top doesn't go above the box: rule_y >= 0.
    // Body baseline = body_top + body_h. We choose rule_y so that the body
    // clears the rule by `gap`:  body_top = rule_y + rule_thickness + gap.
    // The parent baseline is the body baseline.
    let rule_y = 0.0_f32;
    let body_y = rule_y + rule_thickness + gap;
    let surd_y = rule_y; // glyph top edge meets vinculum top

    let parent_height = body_y + body_h;
    // Depth: whichever descends further below the baseline — radicand or surd.
    let surd_bottom = surd_y + surd_below_rule;
    let body_bottom = body_y + body_h + body_d;
    let parent_depth = (surd_bottom.max(body_bottom) - parent_height).max(0.0);

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
    ];

    children.push(Child {
        offset: Point {
            x: surd_w,
            y: body_y,
        },
        child: body_box,
    });

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
        // Raise the degree so its bottom sits `raise_pct` of the surd's total
        // height above the surd's lowest point. Surd spans [surd_y,
        // surd_y + surd_below_rule] in parent coords.
        let surd_bottom_y = surd_y + surd_below_rule;
        let deg_y = (surd_bottom_y - (surd_below_rule * raise_pct) - deg_h).max(0.0);
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

/// Layout an accent (`\hat`, `\vec`, …): center the combining-accent glyph
/// horizontally over the body and raise it to sit just above the body's top.
fn layout_accent(accent: crate::font::GlyphId, body: &Node, style: Style) -> Box {
    use crate::font::{self, math_constant, MathConstant};

    let body_box = layout(body, style);
    let base_size = style.font_size(approx_base_font_size_from_node(body));

    // Accent glyph geometry. Combining accents have zero advance; their visible
    // width comes from the bounding box.
    let am = font::glyph_metrics(accent, base_size);
    let (ax_min, ax_max) = font::glyph_x_bounds(accent, base_size);
    let accent_w = (ax_max - ax_min).max(0.0);

    // Vertical: the accent's bottom should clear the body top by a small gap,
    // but never sit lower than AccentBaseHeight (tall glyphs keep accents from
    // floating too high). Gap = a thin slice of the em.
    let accent_base = math_constant(MathConstant::AccentBaseHeight, base_size);
    let gap = base_size * 0.06;
    // How high above the body baseline the accent's baseline must sit so the
    // accent glyph bottom clears the body top (or accent_base, whichever lower).
    let body_top_above_baseline = body_box.height;
    let clearance = body_top_above_baseline.max(accent_base) + gap;
    // Accent glyph bottom (am.depth below its baseline). Raise the accent so its
    // *bottom* lands at `clearance` above the body baseline.
    let accent_baseline_rise = clearance + am.depth;

    let width = body_box.width;
    let body_center = width / 2.0;

    // Accent glyph paths carry their own x coordinates (often negative for
    // combining forms). To center the glyph's bbox over the body center, the
    // glyph origin lands at body_center - (bbox midpoint).
    let accent_bbox_center = (ax_min + ax_max) / 2.0;
    let accent_origin_x = body_center - accent_bbox_center;

    let accent_box = Box {
        width: accent_w,
        height: am.height,
        depth: am.depth,
        kind: BoxKind::Glyph {
            glyph_id: accent,
            font_size: base_size,
        },
    };

    // y-down: parent baseline at y = parent_height. A child at offset.y has its
    // glyph baseline at offset.y + child.height.
    let body_depth = body_box.depth;
    let body_h = body_box.height;
    let parent_height = body_h.max(accent_baseline_rise + am.height);
    let body_y = parent_height - body_h;
    let accent_y = parent_height - accent_baseline_rise - am.height;

    let children = vec![
        Child {
            offset: Point { x: 0.0, y: body_y },
            child: body_box,
        },
        Child {
            offset: Point {
                x: accent_origin_x,
                y: accent_y,
            },
            child: accent_box,
        },
    ];

    Box {
        width,
        height: parent_height,
        depth: body_depth,
        kind: BoxKind::HBox(children),
    }
}

/// Layout a matrix / aligned / cases grid. Computes per-column widths and
/// per-row asc/descent, places each cell with its column alignment, and centers
/// the whole grid vertically on the math axis so it lines up with neighbours.
fn layout_matrix(rows: &[Vec<Node>], col_aligns: &[crate::ir::ColAlign], style: Style) -> Box {
    use crate::font::{math_constant, MathConstant};
    use crate::ir::ColAlign;

    if rows.is_empty() {
        return Box {
            width: 0.0,
            height: 0.0,
            depth: 0.0,
            kind: BoxKind::Empty,
        };
    }

    // Lay out every cell; track grid dimensions.
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let nrows = rows.len();
    if ncols == 0 {
        return Box {
            width: 0.0,
            height: 0.0,
            depth: 0.0,
            kind: BoxKind::Empty,
        };
    }

    let mut cells: Vec<Vec<Option<Box>>> = Vec::with_capacity(nrows);
    let mut col_w = vec![0.0f32; ncols];
    let mut row_h = vec![0.0f32; nrows];
    let mut row_d = vec![0.0f32; nrows];

    let base_size = style.font_size(approx_base_font_size_from_node_matrix(rows));
    // Matrix cells render in text style (not display) regardless of context.
    let cell_style = match style {
        Style::Display => Style::Text,
        s => s,
    };

    for (r, row) in rows.iter().enumerate() {
        let mut out_row: Vec<Option<Box>> = Vec::with_capacity(ncols);
        for (c, col_w_c) in col_w.iter_mut().enumerate() {
            if let Some(node) = row.get(c) {
                let b = layout(node, cell_style);
                *col_w_c = col_w_c.max(b.width);
                row_h[r] = row_h[r].max(b.height);
                row_d[r] = row_d[r].max(b.depth);
                out_row.push(Some(b));
            } else {
                out_row.push(None);
            }
        }
        cells.push(out_row);
    }

    // Spacing: a column gap (~em) and a row gap (~0.4em).
    let col_gap = base_size * 0.8;
    let row_gap = base_size * 0.4;

    let total_w: f32 = col_w.iter().sum::<f32>() + col_gap * (ncols.saturating_sub(1)) as f32;

    // Stack rows top→bottom; record each row's baseline y (from grid top).
    let mut row_baseline = vec![0.0f32; nrows];
    let mut y = 0.0f32;
    for r in 0..nrows {
        y += row_h[r];
        row_baseline[r] = y;
        y += row_d[r];
        if r + 1 < nrows {
            y += row_gap;
        }
    }
    let total_h = y;

    // Center the grid on the math axis: the grid's vertical midpoint should sit
    // at the axis height above the parent baseline.
    let axis = math_constant(MathConstant::AxisHeight, base_size);
    let half = total_h / 2.0;
    let height = half + axis;
    let depth = half - axis;

    // Column left edges.
    let mut col_x = vec![0.0f32; ncols];
    let mut x = 0.0;
    for (slot, w) in col_x.iter_mut().zip(col_w.iter()) {
        *slot = x;
        x += w + col_gap;
    }

    let mut children = Vec::new();
    for (r, out_row) in cells.into_iter().enumerate() {
        for (c, maybe) in out_row.into_iter().enumerate() {
            let Some(b) = maybe else { continue };
            let align = col_aligns.get(c).copied().unwrap_or(ColAlign::Center);
            let cell_x = match align {
                ColAlign::Left => col_x[c],
                ColAlign::Right => col_x[c] + (col_w[c] - b.width),
                ColAlign::Center => col_x[c] + (col_w[c] - b.width) / 2.0,
            };
            // Cell baseline aligns with the row baseline; box top sits at
            // (row_baseline - b.height) measured from grid top.
            let cell_y = row_baseline[r] - b.height;
            children.push(Child {
                offset: Point {
                    x: cell_x,
                    y: cell_y,
                },
                child: b,
            });
        }
    }

    Box {
        width: total_w,
        height,
        depth,
        kind: BoxKind::HBox(children),
    }
}

/// Best-effort base font size for a matrix: first non-empty cell.
fn approx_base_font_size_from_node_matrix(rows: &[Vec<Node>]) -> f32 {
    for row in rows {
        for cell in row {
            let s = approx_base_font_size_from_node(cell);
            if s > 0.0 {
                return s;
            }
        }
    }
    16.0
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

    // Big ops are a single glyph (layout_op); named ops are a row of letters.
    let b = match base {
        Node::OpName { body, .. } => layout(body, style),
        _ => layout_op(base, style),
    };
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
fn layout_frac(num: &Node, den: &Node, bar: bool, style: Style) -> Box {
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

    let (shift_up_pref, shift_down_pref, num_gap_min, den_gap_min) = if style.is_display() {
        (
            math_constant(MathConstant::FractionNumDisplayStyleShiftUp, base),
            math_constant(MathConstant::FractionDenomDisplayStyleShiftDown, base),
            math_constant(MathConstant::FractionNumDisplayStyleGapMin, base),
            math_constant(MathConstant::FractionDenomDisplayStyleGapMin, base),
        )
    } else {
        (
            math_constant(MathConstant::FractionNumeratorShiftUp, base),
            math_constant(MathConstant::FractionDenominatorShiftDown, base),
            math_constant(MathConstant::FractionNumeratorGapMin, base),
            math_constant(MathConstant::FractionDenominatorGapMin, base),
        )
    };

    // Position num/den by their *ink* relative to the rule, not by the
    // font's ordinary-baseline preferred shift. The preferred shifts
    // (FractionNumeratorShiftUp etc.) assume a numerator with normal descent;
    // a bare digit has zero depth, so honouring the full preferred shift
    // floats it far above the rule while the GapMin clamp pins the
    // denominator close — an asymmetric, lopsided fraction.
    //
    // Instead: clamp each side so the ink clears the rule by GapMin, and use
    // the preferred shift only as an *upper* bound on how far the baseline may
    // sit from the axis. Cap that bound at the symmetric ink gap so both gaps
    // stay balanced. Tall content (radical numerator) still clears via the
    // GapMin floor.
    let half_rule = rule_thickness / 2.0;
    // Numerator baseline sits `shift_up` above the rule axis; the numerator's
    // ink bottom is `shift_up - n.depth` above it. We want that ink bottom a
    // GapMin above the rule's top edge, so the *baseline* shift is
    // `n.depth + half_rule + num_gap_min`. Symmetrically for the denominator.
    // The font's ShiftUp/ShiftDown act only as a lower bound (so ordinary
    // fractions keep the font's optical baseline placement). For a bare digit
    // (n.depth = 0) the font ShiftUp is far larger than needed and would float
    // the numerator high above a denominator pinned by its own GapMin clamp —
    // so cap the numerator shift at the denominator's required shift to keep
    // the rule visually centred between the two glyphs.
    let num_min = n.depth + half_rule + num_gap_min;
    let den_min = d.height + half_rule + den_gap_min;
    let shift_down = den_min.max(shift_down_pref);
    // Mirror the denominator's gap above the rule for the numerator: the gap
    // between rule-top and numerator-ink-bottom should match the gap between
    // rule-bottom and denominator-ink-top, unless the numerator's own descent
    // forces it higher.
    let den_gap_above_ink = (shift_down - d.height) - half_rule; // = effective den gap
    let shift_up = num_min
        .max(n.depth + half_rule + den_gap_above_ink)
        .min(shift_up_pref.max(num_min));

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
    // Horizontal overhang: the rule extends slightly past the wider of
    // num/den on each side, ensuring the bar remains visible (and visually
    // anchored) even when both children are very narrow (e.g. \frac{1}{2}).
    // KaTeX uses ~0.05em of nulldelimiterspace-ish padding per side; we
    // approximate with 2× rule thickness, which scales naturally with style.
    let overhang = rule_thickness * 2.0;
    let content_width = n.width.max(d.width);
    let width = content_width + 2.0 * overhang;

    let num_x = overhang + (content_width - n.width) / 2.0;
    let den_x = overhang + (content_width - d.width) / 2.0;

    let axis_y = parent_height;
    let num_baseline_y = axis_y - shift_up;
    let num_top = num_baseline_y - n.height;
    let den_baseline_y = axis_y + shift_down;
    let den_top = den_baseline_y - d.height;
    let rule_top = axis_y - rule_thickness / 2.0;

    let mut children = vec![Child {
        offset: Point {
            x: num_x,
            y: num_top,
        },
        child: n,
    }];
    // `\binom`/`\atop` are ruleless: keep the same axis gaps (so the stack
    // spacing matches `\frac`) but omit the visible rule rectangle.
    if bar {
        children.push(Child {
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
        });
    }
    children.push(Child {
        offset: Point {
            x: den_x,
            y: den_top,
        },
        child: d,
    });

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
        Node::OpName { body, .. } => approx_base_font_size_from_node(body),
        Node::Accent { body, .. } => approx_base_font_size_from_node(body),
        Node::Matrix { rows, .. } => approx_base_font_size_from_node_matrix(rows),
        _ => 16.0,
    }
}

fn atom_class(node: &Node) -> Option<crate::ir::AtomClass> {
    match node {
        Node::Atom { class, .. } => Some(*class),
        Node::Op { .. } | Node::OpName { .. } => Some(crate::ir::AtomClass::Op),
        Node::Fenced { .. } => Some(crate::ir::AtomClass::Inner),
        Node::Frac { .. }
        | Node::Radical { .. }
        | Node::Subsup { .. }
        | Node::Row(_)
        | Node::Accent { .. }
        | Node::Matrix { .. } => Some(crate::ir::AtomClass::Ord),
        _ => None,
    }
}

fn approx_font_size(node: &Node) -> f32 {
    match node {
        Node::Atom { font_size, .. } | Node::Op { font_size, .. } => *font_size,
        _ => 16.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Style;
    use crate::parse;

    // --- boxer_atom.rs ---
    #[test]
    fn single_atom_box_has_expected_width() {
        let ir = parse::to_ir("x", 16.0, Style::Text).unwrap();
        let b = layout(&ir, Style::Text);
        assert!(b.width > 0.0, "x must have positive width");
        assert!(b.height > 0.0, "x must have positive height");
    }

    #[test]
    fn ab_is_wider_than_a() {
        let a = layout(&parse::to_ir("a", 16.0, Style::Text).unwrap(), Style::Text);
        let ab = layout(&parse::to_ir("ab", 16.0, Style::Text).unwrap(), Style::Text);
        assert!(ab.width > a.width);
    }

    #[test]
    fn aplusb_wider_than_ab_due_to_med_spacing() {
        let ab = layout(&parse::to_ir("ab", 16.0, Style::Text).unwrap(), Style::Text);
        let aplus = layout(
            &parse::to_ir("a+b", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            aplus.width > ab.width,
            "a+b should be wider than ab due to Med spacing around +"
        );
    }

    // --- boxer_bigop.rs ---
    #[test]
    fn sum_display_has_limits_above_below() {
        let bare = layout(
            &parse::to_ir(r"\sum", 16.0, Style::Display).unwrap(),
            Style::Display,
        );
        let s = layout(
            &parse::to_ir(r"\sum_{i=1}^{n}", 16.0, Style::Display).unwrap(),
            Style::Display,
        );
        assert!(
            s.height > bare.height,
            "display sum with sup limit should be taller than bare sum: bare.h={} sum.h={}",
            bare.height,
            s.height,
        );
        assert!(
            s.depth > bare.depth,
            "display sum with sub limit should have more depth than bare sum: bare.d={} sum.d={}",
            bare.depth,
            s.depth,
        );
    }

    // --- boxer_fenced.rs ---
    #[test]
    fn fenced_around_frac_uses_taller_paren_variant_than_inline_paren() {
        let inline = layout(
            &parse::to_ir("(x)", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let fenced = layout(
            &parse::to_ir(r"\left(\frac{1}{2}\right)", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            fenced.height > inline.height,
            "left/right should size to body; got fenced.height={} inline.height={}",
            fenced.height,
            inline.height
        );
    }

    // --- boxer_frac.rs ---
    #[test]
    fn frac_height_exceeds_num_height_alone() {
        let half = layout(
            &parse::to_ir(r"\frac{1}{2}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let one = layout(&parse::to_ir("1", 16.0, Style::Text).unwrap(), Style::Text);
        assert!(
            half.height + half.depth > one.height,
            "frac total extent should exceed just the numerator"
        );
        assert!(
            half.depth > 0.0,
            "frac should have nonzero depth (denominator below axis)"
        );
    }

    #[test]
    fn frac_width_at_least_max_of_num_den() {
        let f = layout(
            &parse::to_ir(r"\frac{abc}{de}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let num = layout(
            &parse::to_ir("abc", 16.0, Style::Text).unwrap(),
            Style::Script,
        );
        let den = layout(
            &parse::to_ir("de", 16.0, Style::Text).unwrap(),
            Style::Script,
        );
        assert!(
            f.width >= num.width.max(den.width),
            "frac width {} should be >= max(num_script {}, den_script {})",
            f.width,
            num.width,
            den.width,
        );
    }

    #[test]
    fn frac_baseline_at_axis_sanity() {
        let f = layout(
            &parse::to_ir(r"\frac{1}{2}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            f.height + f.depth > 16.0,
            "frac total ink height should exceed 1em, got {} + {} = {}",
            f.height,
            f.depth,
            f.height + f.depth
        );
    }

    #[test]
    fn row_baselines_align() {
        let ir = parse::to_ir("a", 16.0, Style::Text).unwrap();
        let b = layout(&ir, Style::Text);
        let BoxKind::HBox(children) = &b.kind else {
            panic!("expected HBox at row top, got {:?}", b.kind);
        };
        for c in children {
            let expected_y = b.height - c.child.height;
            assert!(
                (c.offset.y - expected_y).abs() < 0.01,
                "child top offset should equal parent.height - child.height; got {} expected {}",
                c.offset.y,
                expected_y
            );
        }
    }

    // --- boxer_radical.rs ---
    #[test]
    fn sqrt_height_exceeds_body_height() {
        let x = layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
        let s = layout(
            &parse::to_ir(r"\sqrt{x}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(s.height > x.height);
    }

    #[test]
    fn sqrt_with_long_degree_widens_box() {
        let s_normal = layout(
            &parse::to_ir(r"\sqrt{x}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let s_long = layout(
            &parse::to_ir(r"\sqrt[12345]{x}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            s_long.width > s_normal.width + 10.0,
            "wide degree must widen the parent box: long={} normal={}",
            s_long.width,
            s_normal.width,
        );
    }

    // --- boxer_subsup.rs ---
    #[test]
    fn script_glyph_size_smaller_than_base() {
        let bx = layout(
            &parse::to_ir("x^2", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let BoxKind::HBox(top_children) = &bx.kind else {
            panic!("expected HBox at top, got {:?}", bx.kind);
        };
        let subsup = &top_children
            .last()
            .expect("at least one child at top")
            .child;
        let BoxKind::HBox(inner) = &subsup.kind else {
            panic!("expected HBox (subsup), got {:?}", subsup.kind);
        };
        let sup_child = inner.last().expect("subsup should have at least one child");
        let BoxKind::Glyph { font_size, .. } = sup_child.child.kind else {
            panic!("expected glyph for sup, got {:?}", sup_child.child.kind);
        };
        assert!(
            font_size < 16.0,
            "sup font_size should be < base 16; got {}",
            font_size
        );
        assert!(
            (font_size - 16.0 * 0.7).abs() < 0.01,
            "expected ~11.2 (16 * 0.7), got {}",
            font_size
        );
    }

    #[test]
    fn xsup2_taller_than_x() {
        let x = layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
        let xs = layout(
            &parse::to_ir("x^2", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(xs.height > x.height);
    }

    #[test]
    fn xsub_has_more_depth_than_x() {
        let x = layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
        let xs = layout(
            &parse::to_ir("x_i", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(xs.depth > x.depth);
    }

    // --- boxer_functions.rs ---
    #[test]
    fn sin_letters_have_no_inter_letter_spacing() {
        // \sin as one upright op-name unit must be exactly as wide as the bare
        // glyph row "sin" (Ord letters) — no thin Op-Op spacing between letters.
        let func = layout(
            &parse::to_ir(r"\sin", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let plain = layout(
            &parse::to_ir("sin", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            (func.width - plain.width).abs() < 0.01,
            "\\sin width {} must equal plain 'sin' width {} (no inter-letter spacing)",
            func.width,
            plain.width
        );
    }

    #[test]
    fn sin_applies_op_spacing_to_following_arg() {
        // \sin x should be wider than the glyphs "sinx" set tight, because the
        // op-name unit gets Op->Ord thin spacing before its argument.
        let with_op = layout(
            &parse::to_ir(r"\sin x", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let tight = layout(
            &parse::to_ir("sinx", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            with_op.width > tight.width,
            "\\sin x ({}) should exceed tight 'sinx' ({}) by Op spacing",
            with_op.width,
            tight.width
        );
    }

    // --- boxer_matrix.rs ---
    #[test]
    fn matrix_grid_has_rows_and_cols() {
        let b = layout(
            &parse::to_ir(
                r"\begin{matrix} a & b \\ c & d \end{matrix}",
                16.0,
                Style::Text,
            )
            .unwrap(),
            Style::Text,
        );
        // Grid renders with positive extent and four glyph leaves (a,b,c,d).
        assert!(b.width > 0.0 && b.height > 0.0 && b.depth > 0.0);
        assert_eq!(count_glyphs(&b), 4, "2x2 matrix has 4 glyph cells");
    }

    fn count_glyphs(b: &Box) -> usize {
        match &b.kind {
            BoxKind::Glyph { .. } => 1,
            BoxKind::HBox(c) | BoxKind::VBox(c) => c.iter().map(|ch| count_glyphs(&ch.child)).sum(),
            _ => 0,
        }
    }

    #[test]
    fn matrix_wider_with_more_columns() {
        let two = layout(
            &parse::to_ir(r"\begin{matrix} a & b \end{matrix}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let three = layout(
            &parse::to_ir(r"\begin{matrix} a & b & c \end{matrix}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            three.width > two.width,
            "3 cols ({}) wider than 2 ({})",
            three.width,
            two.width
        );
    }

    #[test]
    fn matrix_taller_with_more_rows() {
        let one = layout(
            &parse::to_ir(r"\begin{matrix} a \end{matrix}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        let two = layout(
            &parse::to_ir(r"\begin{matrix} a \\ b \end{matrix}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            two.height + two.depth > one.height + one.depth,
            "2 rows ({}) taller than 1 ({})",
            two.height + two.depth,
            one.height + one.depth
        );
    }

    // --- boxer_accents.rs ---
    #[test]
    fn hat_is_taller_than_bare_x() {
        let x = layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
        let hat = layout(
            &parse::to_ir(r"\hat{x}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            hat.height > x.height,
            "\\hat{{x}} ({}) must be taller than bare x ({}) — accent sits above",
            hat.height,
            x.height
        );
    }

    #[test]
    fn accent_does_not_change_body_width() {
        // Accent is centered over the body; it must not widen the box.
        let x = layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
        let hat = layout(
            &parse::to_ir(r"\hat{x}", 16.0, Style::Text).unwrap(),
            Style::Text,
        );
        assert!(
            (hat.width - x.width).abs() < 0.01,
            "accent width {} should equal body width {}",
            hat.width,
            x.width
        );
    }

    #[test]
    fn lim_subscript_stacks_underneath_in_display() {
        let bare = layout(
            &parse::to_ir(r"\lim", 16.0, Style::Display).unwrap(),
            Style::Display,
        );
        let stacked = layout(
            &parse::to_ir(r"\lim_{x \to 0}", 16.0, Style::Display).unwrap(),
            Style::Display,
        );
        assert!(
            stacked.depth > bare.depth,
            "\\lim with display subscript must stack underneath (more depth): bare.d={} lim.d={}",
            bare.depth,
            stacked.depth
        );
    }
}
