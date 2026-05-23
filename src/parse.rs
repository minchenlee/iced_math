//! pulldown-latex events → IR.
//!
//! v0.1 Task 10: atoms.
//! v0.1 Task 11: sub/superscripts with group-aware element reading.

use pulldown_latex::event::{Content, DelimiterType, Event, Grouping, ScriptType};
use pulldown_latex::{Parser, Storage};

use crate::font;
use crate::ir::{AtomClass, Node, Style};

#[derive(Debug)]
pub struct ParseError(pub String);

pub fn to_ir(src: &str, font_size: f32, style: Style) -> Result<Node, ParseError> {
    let storage = Storage::new();
    let parser = Parser::new(src, &storage);

    // Collect into a Vec so we can drive a manual index cursor (needed for
    // Script element-by-element consumption).
    let mut events = Vec::new();
    for ev in parser {
        events.push(ev.map_err(|e| ParseError(format!("{e:?}")))?);
    }

    let mut cursor = 0usize;
    let row = parse_until_end(&events, &mut cursor, font_size, style, /* in_group */ false)?;
    Ok(Node::Row(row))
}

/// Parse events into a row vector, stopping at top-level end of stream or
/// (when `in_group`) at the matching `Event::End`. Consumes the terminating
/// `End` event when `in_group` is true.
fn parse_until_end(
    events: &[Event],
    cursor: &mut usize,
    font_size: f32,
    style: Style,
    in_group: bool,
) -> Result<Vec<Node>, ParseError> {
    let mut row = Vec::new();
    while *cursor < events.len() {
        if in_group {
            if let Event::End = events[*cursor] {
                *cursor += 1;
                return Ok(row);
            }
        }
        let node = parse_element(events, cursor, font_size, style)?;
        if let Some(n) = node {
            row.push(n);
        }
    }
    if in_group {
        return Err(ParseError("unterminated group (missing End)".into()));
    }
    Ok(row)
}

/// Consume one "element" from the event stream. May return `None` for events
/// that are silently ignored at v0.1 (state changes, env flow, etc.).
fn parse_element(
    events: &[Event],
    cursor: &mut usize,
    font_size: f32,
    style: Style,
) -> Result<Option<Node>, ParseError> {
    if *cursor >= events.len() {
        return Err(ParseError("expected element, got end of stream".into()));
    }
    let ev = events[*cursor].clone();
    *cursor += 1;
    match ev {
        Event::Content(c) => Ok(Some(content_to_node(c, font_size, style)?)),
        Event::Begin(Grouping::Normal) => {
            let inner = parse_until_end(events, cursor, font_size, style, /* in_group */ true)?;
            Ok(Some(Node::Row(inner)))
        }
        Event::Begin(_) => {
            // Other groupings (LeftRight, environments) — handled in later tasks.
            // For now, consume to matching End to keep stream balanced.
            let inner = parse_until_end(events, cursor, font_size, style, /* in_group */ true)?;
            Ok(Some(Node::Row(inner)))
        }
        Event::End => Err(ParseError("unexpected End outside group".into())),
        Event::Script { ty, .. } => {
            let base = parse_element(events, cursor, font_size, style)?
                .ok_or_else(|| ParseError("script base produced no node".into()))?;
            let (sub, sup) = match ty {
                ScriptType::Subscript => {
                    let s = parse_element(events, cursor, font_size, style)?
                        .ok_or_else(|| ParseError("subscript produced no node".into()))?;
                    (Some(Box::new(s)), None)
                }
                ScriptType::Superscript => {
                    let s = parse_element(events, cursor, font_size, style)?
                        .ok_or_else(|| ParseError("superscript produced no node".into()))?;
                    (None, Some(Box::new(s)))
                }
                ScriptType::SubSuperscript => {
                    let sb = parse_element(events, cursor, font_size, style)?
                        .ok_or_else(|| ParseError("subscript produced no node".into()))?;
                    let sp = parse_element(events, cursor, font_size, style)?
                        .ok_or_else(|| ParseError("superscript produced no node".into()))?;
                    (Some(Box::new(sb)), Some(Box::new(sp)))
                }
            };
            Ok(Some(Node::Subsup { base: Box::new(base), sub, sup }))
        }
        // v0.1 future tasks: Visual (frac/sqrt/negation), Space, StateChange, EnvironmentFlow.
        // Consume but produce nothing for now.
        Event::Visual(_) | Event::Space { .. } | Event::StateChange(_) | Event::EnvironmentFlow(_) => {
            Ok(None)
        }
    }
}

fn content_to_node(c: Content, font_size: f32, style: Style) -> Result<Node, ParseError> {
    let size = style.font_size(font_size);
    // Most content variants produce a single atom; string-bearing variants
    // (Number, Text, Function) wrap multiple atoms in a Row.
    match c {
        Content::Ordinary { content, .. } => atom_node(content, AtomClass::Ord, size),
        Content::Number(s) => chars_to_node(s.chars(), AtomClass::Ord, size),
        Content::Text(s) => chars_to_node(s.chars(), AtomClass::Ord, size),
        Content::Function(s) => chars_to_node(s.chars(), AtomClass::Op, size),
        Content::BinaryOp { content, .. } => atom_node(content, AtomClass::Bin, size),
        Content::Relation { content, .. } => {
            let mut buf = [0u8; 8];
            let bytes = content.encode_utf8_to_buf(&mut buf);
            let s = std::str::from_utf8(bytes)
                .map_err(|e| ParseError(format!("relation utf8: {e}")))?;
            chars_to_node(s.chars(), AtomClass::Rel, size)
        }
        Content::Delimiter { content, ty, .. } => {
            let class = match ty {
                DelimiterType::Open => AtomClass::Open,
                DelimiterType::Close => AtomClass::Close,
                DelimiterType::Fence => AtomClass::Inner,
            };
            atom_node(content, class, size)
        }
        Content::Punctuation(ch) => atom_node(ch, AtomClass::Punct, size),
        Content::LargeOp { content, .. } => atom_node(content, AtomClass::Op, size),
    }
}

fn atom_node(ch: char, class: AtomClass, font_size: f32) -> Result<Node, ParseError> {
    let glyph = font::glyph_id(ch)
        .ok_or_else(|| ParseError(format!("no glyph for {ch:?} (U+{:04X})", ch as u32)))?;
    Ok(Node::Atom { class, glyph, font_size })
}

fn chars_to_node<I: Iterator<Item = char>>(
    chars: I,
    class: AtomClass,
    font_size: f32,
) -> Result<Node, ParseError> {
    let mut nodes = Vec::new();
    for ch in chars {
        nodes.push(atom_node(ch, class, font_size)?);
    }
    if nodes.len() == 1 {
        Ok(nodes.into_iter().next().unwrap())
    } else {
        Ok(Node::Row(nodes))
    }
}
