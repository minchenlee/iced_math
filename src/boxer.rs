//! IR → positioned Box tree.

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
        Node::Error(_) => Box { width: 0.0, height: 0.0, depth: 0.0, kind: BoxKind::Empty },
        _ => Box { width: 0.0, height: 0.0, depth: 0.0, kind: BoxKind::Empty },
    }
}

fn layout_row(items: &[Node], style: Style) -> Box {
    use crate::spacing;
    let mut children = Vec::new();
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
        children.push(Child { offset: Point { x: cursor, y: 0.0 }, child: b });
        cursor += width;
        prev_class = cur_class;
    }
    Box { width: cursor, height, depth, kind: BoxKind::HBox(children) }
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
