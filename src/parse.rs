//! pulldown-latex events → IR.
//!
//! v0.1 Task 10: atoms only. Sub/sup, frac, sqrt, delimiters come in later tasks.

use pulldown_latex::event::{Content, DelimiterType, Event};
use pulldown_latex::{Parser, Storage};

use crate::font;
use crate::ir::{AtomClass, Node, Style};

#[derive(Debug)]
pub struct ParseError(pub String);

pub fn to_ir(src: &str, font_size: f32, style: Style) -> Result<Node, ParseError> {
    let storage = Storage::new();
    let parser = Parser::new(src, &storage);

    let mut row = Vec::new();
    for ev in parser {
        let ev = ev.map_err(|e| ParseError(format!("{e:?}")))?;
        handle_event(ev, &mut row, font_size, style)?;
    }
    Ok(Node::Row(row))
}

fn handle_event(
    ev: Event,
    out: &mut Vec<Node>,
    font_size: f32,
    style: Style,
) -> Result<(), ParseError> {
    let size = style.font_size(font_size);
    match ev {
        Event::Content(c) => match c {
            Content::Ordinary { content, .. } => {
                push_atom(out, content, AtomClass::Ord, size)?;
            }
            Content::Number(s) => {
                for ch in s.chars() {
                    push_atom(out, ch, AtomClass::Ord, size)?;
                }
            }
            Content::Text(s) => {
                for ch in s.chars() {
                    push_atom(out, ch, AtomClass::Ord, size)?;
                }
            }
            Content::Function(s) => {
                for ch in s.chars() {
                    push_atom(out, ch, AtomClass::Op, size)?;
                }
            }
            Content::BinaryOp { content, .. } => {
                push_atom(out, content, AtomClass::Bin, size)?;
            }
            Content::Relation { content, .. } => {
                let mut buf = [0u8; 8];
                let bytes = content.encode_utf8_to_buf(&mut buf);
                let s = std::str::from_utf8(bytes)
                    .map_err(|e| ParseError(format!("relation utf8: {e}")))?;
                for ch in s.chars() {
                    push_atom(out, ch, AtomClass::Rel, size)?;
                }
            }
            Content::Delimiter { content, ty, .. } => {
                let class = match ty {
                    DelimiterType::Open => AtomClass::Open,
                    DelimiterType::Close => AtomClass::Close,
                    DelimiterType::Fence => AtomClass::Inner,
                };
                push_atom(out, content, class, size)?;
            }
            Content::Punctuation(ch) => {
                push_atom(out, ch, AtomClass::Punct, size)?;
            }
            // LargeOp: handled fully in Task 13. For now, treat as Op atom so tests don't blow up.
            Content::LargeOp { content, .. } => {
                push_atom(out, content, AtomClass::Op, size)?;
            }
        },
        // Non-atom events are skipped in Task 10; later tasks will handle them.
        _ => {}
    }
    Ok(())
}

fn push_atom(
    out: &mut Vec<Node>,
    ch: char,
    class: AtomClass,
    font_size: f32,
) -> Result<(), ParseError> {
    let glyph = font::glyph_id(ch)
        .ok_or_else(|| ParseError(format!("no glyph for {ch:?} (U+{:04X})", ch as u32)))?;
    out.push(Node::Atom { class, glyph, font_size });
    Ok(())
}
