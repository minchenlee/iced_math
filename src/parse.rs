//! pulldown-latex events → IR.
//!
//! v0.1 Task 10: atoms.
//! v0.1 Task 11: sub/superscripts with group-aware element reading.

use pulldown_latex::event::{Content, DelimiterType, Event, Grouping, ScriptType, Visual};
use pulldown_latex::{Parser, Storage};
use ttf_parser::GlyphId;

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
    let row = parse_until_end(
        &events,
        &mut cursor,
        font_size,
        style,
        /* in_group */ false,
    )?;
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
            let inner =
                parse_until_end(events, cursor, font_size, style, /* in_group */ true)?;
            Ok(Some(Node::Row(inner)))
        }
        Event::Begin(Grouping::LeftRight(open_opt, close_opt)) => {
            let inner =
                parse_until_end(events, cursor, font_size, style, /* in_group */ true)?;
            // v0.1: base glyph IDs only — boxer will swap to vertical variants
            // sized to body height. `\left.`/`\right.` (None) → GlyphId(0) sentinel,
            // which the boxer treats as no-op (invisible delimiter).
            let open = delim_glyph(open_opt);
            let close = delim_glyph(close_opt);
            Ok(Some(Node::Fenced {
                open,
                close,
                body: Box::new(Node::Row(inner)),
            }))
        }
        Event::Begin(_) => {
            // Other groupings (environments) — handled in later tasks.
            // For now, consume to matching End to keep stream balanced.
            let inner =
                parse_until_end(events, cursor, font_size, style, /* in_group */ true)?;
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
            Ok(Some(Node::Subsup {
                base: Box::new(base),
                sub,
                sup,
            }))
        }
        Event::Visual(v) => match v {
            Visual::Fraction(_) => {
                let num = parse_element(events, cursor, font_size, style)?
                    .ok_or_else(|| ParseError("fraction numerator produced no node".into()))?;
                let den = parse_element(events, cursor, font_size, style)?
                    .ok_or_else(|| ParseError("fraction denominator produced no node".into()))?;
                Ok(Some(Node::Frac {
                    num: Box::new(num),
                    den: Box::new(den),
                }))
            }
            Visual::SquareRoot => {
                let body = parse_element(events, cursor, font_size, style)?
                    .ok_or_else(|| ParseError("sqrt body produced no node".into()))?;
                Ok(Some(Node::Radical {
                    degree: None,
                    body: Box::new(body),
                }))
            }
            Visual::Root => {
                // pulldown-latex order: radicand then index.
                let body = parse_element(events, cursor, font_size, style)?
                    .ok_or_else(|| ParseError("root radicand produced no node".into()))?;
                let degree = parse_element(events, cursor, font_size, style)?
                    .ok_or_else(|| ParseError("root index produced no node".into()))?;
                Ok(Some(Node::Radical {
                    degree: Some(Box::new(degree)),
                    body: Box::new(body),
                }))
            }
            Visual::Negation => Ok(None),
        },
        // v0.1 future tasks: Space, StateChange, EnvironmentFlow. Consume but produce nothing.
        Event::Space { .. } | Event::StateChange(_) | Event::EnvironmentFlow(_) => Ok(None),
    }
}

fn content_to_node(c: Content, font_size: f32, style: Style) -> Result<Node, ParseError> {
    // Note: we store the BASE font_size on Node::Atom / Node::Op (not
    // style-scaled). The boxer applies `style.font_size(base)` at layout time
    // so script-style atoms actually render at script-scale glyph metrics.
    let size = font_size;
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
        Content::LargeOp { content, small } => large_op_node(content, small, font_size, style),
    }
}

fn large_op_node(ch: char, small: bool, font_size: f32, style: Style) -> Result<Node, ParseError> {
    // Store the BASE font_size; boxer applies style scaling at layout time.
    let size = font_size;
    let base_glyph = font::glyph_id(ch)
        .ok_or_else(|| ParseError(format!("no glyph for {ch:?} (U+{:04X})", ch as u32)))?;
    // `big` = pick the larger MATH variant. Display mode triggers it, unless
    // the operator is forced small (e.g. \smallint, \tsum).
    let big = style.is_display() && !small;
    let glyph = if big {
        font::math_variant_vertical(base_glyph, 1500.0)
            .map(|(g, _)| g)
            .unwrap_or(base_glyph)
    } else {
        base_glyph
    };
    // v0.1: `limits` follows display mode for large ops (matches LaTeX default
    // for \sum, \prod, \int's siblings — \int is conventionally non-limits in
    // display too, but for v0.1 we keep the simple rule).
    let limits = big;
    Ok(Node::Op {
        glyph,
        limits,
        big,
        font_size: size,
    })
}

/// Resolve a `\left`/`\right` delimiter char to a base GlyphId.
/// `None` (LaTeX null delim `.`) and unmapped chars both fall back to
/// `GlyphId(0)`, the sentinel the boxer treats as an invisible delimiter.
fn delim_glyph(ch: Option<char>) -> GlyphId {
    ch.and_then(font::glyph_id).unwrap_or(GlyphId(0))
}

fn atom_node(ch: char, class: AtomClass, font_size: f32) -> Result<Node, ParseError> {
    let glyph = font::glyph_id(ch)
        .ok_or_else(|| ParseError(format!("no glyph for {ch:?} (U+{:04X})", ch as u32)))?;
    Ok(Node::Atom {
        class,
        glyph,
        font_size,
    })
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
