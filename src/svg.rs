//! Box tree → SVG byte stream.

use std::fmt::Write;

use crate::boxer::{Box as MBox, BoxKind, Child, Point};
use crate::font;

pub fn emit(root: &MBox) -> Vec<u8> {
    let w = root.width.max(0.0);
    let h = (root.height + root.depth).max(0.0);
    let mut out = String::new();
    let _ = write!(
        &mut out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = w, h = h
    );
    walk(&mut out, root, Point { x: 0.0, y: 0.0 });
    out.push_str("</svg>");
    out.into_bytes()
}

fn walk(out: &mut String, b: &MBox, origin: Point) {
    match &b.kind {
        BoxKind::Glyph { glyph_id, font_size } => {
            let s = font_size / font::units_per_em();
            let path_d = font::outline_path(*glyph_id);
            if path_d.is_empty() {
                return;
            }
            // Glyph's baseline sits at y = origin.y + b.height (parent baseline at y=height in y-down).
            // SVG path data is in y-up font design units. Transform matrix(s 0 0 -s ox oy) maps
            // (px_design, py_design_up) → (ox + s*px, oy + (-s)*py) = (ox + s*px, oy - s*py).
            // With ox = origin.x, oy = baseline_y, point at (0,0) in design space lands at baseline.
            let baseline_y = origin.y + b.height;
            let _ = write!(
                out,
                r#"<path transform="matrix({s} 0 0 {neg_s} {ox} {oy})" d="{d}"/>"#,
                s = s,
                neg_s = -s,
                ox = origin.x,
                oy = baseline_y,
                d = path_d
            );
        }
        BoxKind::Rule { thickness } => {
            let _ = write!(
                out,
                r#"<rect x="{x}" y="{y}" width="{w}" height="{h}"/>"#,
                x = origin.x,
                y = origin.y,
                w = b.width,
                h = thickness
            );
        }
        BoxKind::HBox(children) | BoxKind::VBox(children) => {
            for Child { offset, child } in children {
                walk(
                    out,
                    child,
                    Point { x: origin.x + offset.x, y: origin.y + offset.y },
                );
            }
        }
        BoxKind::Empty => {}
    }
}
