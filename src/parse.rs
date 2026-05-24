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
        Event::Script { ty, position } => {
            let base = parse_element(events, cursor, font_size, style)?
                .ok_or_else(|| ParseError("script base produced no node".into()))?;

            // Accents arrive as `Script{Superscript, AboveBelow}` where the
            // "superscript" element is the accent glyph (e.g. `→` for \vec). Turn
            // that into a centered over-accent instead of a raised superscript.
            if matches!(ty, ScriptType::Superscript)
                && matches!(position, pulldown_latex::event::ScriptPosition::AboveBelow)
            {
                if let Some(ch) = peek_accent_char(events, *cursor) {
                    if let Some(accent) = font::accent_glyph(ch) {
                        // Consume the accent glyph event directly. We must NOT route
                        // it through parse_element: the spacing accent chars (e.g.
                        // `‾` U+203E for \bar) have no standalone glyph and would
                        // error in atom_node. We've already remapped to the combining
                        // glyph above.
                        *cursor += 1;
                        return Ok(Some(Node::Accent {
                            accent,
                            body: Box::new(base),
                        }));
                    }
                }
            }

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
        Content::Function(s) => function_node(s, size),
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

/// Peek the character of the accent element at `idx` (the element following an
/// `AboveBelow` superscript). The accent surfaces as a single-character
/// `Content` event; returns its char, or `None` if it isn't a bare char.
fn peek_accent_char(events: &[Event], idx: usize) -> Option<char> {
    match events.get(idx)? {
        Event::Content(Content::Ordinary { content, .. }) => Some(*content),
        Event::Content(Content::BinaryOp { content, .. }) => Some(*content),
        _ => None,
    }
}

/// LaTeX operators that stack their scripts as limits (`\DeclareMathOperator*`).
/// Everything else (`\sin`, `\cos`, `\log`, …) attaches scripts to the side.
fn is_limit_op(name: &str) -> bool {
    matches!(
        name,
        "lim" | "limsup" | "liminf" | "max" | "min" | "sup" | "inf" | "det" | "gcd"
            | "Pr" | "argmax" | "argmin"
    )
}

/// Build a multi-letter function name as a single upright operator unit.
/// The letters are `Ord` atoms (no inter-letter math spacing) wrapped in a Row;
/// the `OpName` wrapper gives the whole unit `Op`-class spacing externally.
fn function_node(name: &str, font_size: f32) -> Result<Node, ParseError> {
    let mut letters = Vec::new();
    for ch in name.chars() {
        letters.push(atom_node(ch, AtomClass::Ord, font_size)?);
    }
    let body = if letters.len() == 1 {
        letters.into_iter().next().unwrap()
    } else {
        Node::Row(letters)
    };
    Ok(Node::OpName {
        body: Box::new(body),
        limits: is_limit_op(name),
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{AtomClass, Node, Style};

    // --- parse_atom.rs ---
    #[test]
    fn parses_single_letter() {
        let ir = to_ir("x", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else {
            panic!("expected Row, got {:?}", ir)
        };
        assert_eq!(items.len(), 1);
        let Node::Atom { class, .. } = &items[0] else {
            panic!()
        };
        assert_eq!(*class, AtomClass::Ord);
    }

    #[test]
    fn parses_two_letters_as_row() {
        let ir = to_ir("xy", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn classifies_plus_as_bin() {
        let ir = to_ir("a+b", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Atom { class: c2, .. } = &items[1] else {
            panic!()
        };
        assert_eq!(*c2, AtomClass::Bin);
    }

    // --- parse_fenced.rs ---
    #[test]
    fn parses_left_right_paren() {
        let ir = to_ir(r"\left( x \right)", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        assert_eq!(items.len(), 1);
        let Node::Fenced {
            open: _,
            close: _,
            body,
        } = &items[0]
        else {
            panic!("expected Fenced")
        };
        assert!(matches!(body.as_ref(), Node::Row(_)));
    }

    #[test]
    fn parses_left_right_brackets() {
        let ir = to_ir(r"\left[ \frac{a}{b} \right]", 16.0, Style::Display).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Fenced { .. } = &items[0] else {
            panic!("expected Fenced")
        };
    }

    #[test]
    fn parses_left_dot_null_delim() {
        let ir = to_ir(r"\left. x \right)", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Fenced { open, .. } = &items[0] else {
            panic!("expected Fenced")
        };
        assert_eq!(open.0, 0);
    }

    // --- parse_frac_sqrt.rs ---
    #[test]
    fn parses_frac() {
        let ir = to_ir(r"\frac{1}{2}", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        assert_eq!(items.len(), 1);
        let Node::Frac { num, den } = &items[0] else {
            panic!("expected Frac")
        };
        assert!(matches!(num.as_ref(), Node::Row(_)));
        assert!(matches!(den.as_ref(), Node::Row(_)));
    }

    #[test]
    fn parses_sqrt() {
        let ir = to_ir(r"\sqrt{x}", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Radical { degree, body } = &items[0] else {
            panic!("expected Radical")
        };
        assert!(degree.is_none());
        assert!(matches!(body.as_ref(), Node::Row(_)));
    }

    #[test]
    fn parses_sqrt_with_degree() {
        let ir = to_ir(r"\sqrt[3]{x}", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Radical {
            degree: Some(_),
            body: _,
        } = &items[0]
        else {
            panic!()
        };
    }

    // --- parse_greek_ops.rs ---
    #[test]
    fn parses_alpha() {
        let ir = to_ir(r"\alpha", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        assert_eq!(items.len(), 1);
        let Node::Atom { class, .. } = &items[0] else {
            panic!()
        };
        assert_eq!(*class, AtomClass::Ord);
    }

    #[test]
    fn parses_capital_gamma() {
        let ir = to_ir(r"\Gamma", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn parses_sum_with_limits_in_display() {
        let ir = to_ir(r"\sum_{i=1}^{n}", 16.0, Style::Display).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Subsup {
            base,
            sub: Some(_),
            sup: Some(_),
        } = &items[0]
        else {
            panic!("expected Subsup wrapping Op")
        };
        let Node::Op { limits, big, .. } = base.as_ref() else {
            panic!("expected Op base")
        };
        assert!(*limits, "\\sum in display mode must have limits=true");
        assert!(*big, "\\sum should pick big variant in display");
    }

    #[test]
    fn parses_sum_inline_uses_scripts_not_limits() {
        let ir = to_ir(r"\sum_{i=1}^{n}", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Subsup { base, .. } = &items[0] else {
            panic!()
        };
        let Node::Op { limits, big, .. } = base.as_ref() else {
            panic!()
        };
        assert!(
            !*limits,
            "\\sum in text mode must have limits=false (scripts)"
        );
        assert!(!*big, "\\sum should NOT pick big variant in text");
    }

    // --- parse_subsup.rs ---
    #[test]
    fn parses_superscript() {
        let ir = to_ir("x^2", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        assert_eq!(items.len(), 1);
        let Node::Subsup { sub, sup, .. } = &items[0] else {
            panic!("expected Subsup, got {:?}", items[0])
        };
        assert!(sub.is_none());
        assert!(sup.is_some());
    }

    #[test]
    fn parses_subscript() {
        let ir = to_ir("a_i", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Subsup { sub, sup, .. } = &items[0] else {
            panic!()
        };
        assert!(sub.is_some());
        assert!(sup.is_none());
    }

    #[test]
    fn parses_both_sub_and_sup() {
        let ir = to_ir("a_i^j", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Subsup { sub, sup, .. } = &items[0] else {
            panic!()
        };
        assert!(sub.is_some() && sup.is_some());
    }

    #[test]
    fn parses_braced_exponent() {
        let ir = to_ir("x^{n+1}", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Subsup { sup: Some(sup), .. } = &items[0] else {
            panic!()
        };
        let Node::Row(inner) = sup.as_ref() else {
            panic!("expected Row inside exponent, got {:?}", sup)
        };
        assert_eq!(inner.len(), 3, "n + 1 = 3 atoms");
    }

    // --- parse_functions.rs ---
    #[test]
    fn sin_is_opname_non_limits_with_ord_letters() {
        let ir = to_ir(r"\sin", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::OpName { body, limits } = &items[0] else {
            panic!("expected OpName, got {:?}", items[0])
        };
        assert!(!*limits, "\\sin must not be a limit operator");
        let Node::Row(letters) = body.as_ref() else {
            panic!("expected Row of letters, got {:?}", body)
        };
        assert_eq!(letters.len(), 3, "s i n");
        for l in letters {
            let Node::Atom { class, .. } = l else { panic!() };
            assert_eq!(
                *class,
                AtomClass::Ord,
                "function letters are Ord (no inter-letter Op spacing)"
            );
        }
    }

    #[test]
    fn lim_is_opname_with_limits() {
        let ir = to_ir(r"\lim", 16.0, Style::Display).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::OpName { limits, .. } = &items[0] else {
            panic!("expected OpName, got {:?}", items[0])
        };
        assert!(*limits, "\\lim must be a limit operator");
    }

    // --- parse_accents.rs ---
    #[test]
    fn hat_parses_as_accent_over_body() {
        let ir = to_ir(r"\hat{x}", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Accent { body, .. } = &items[0] else {
            panic!("expected Accent, got {:?}", items[0])
        };
        // body is the Row wrapping x
        assert!(matches!(body.as_ref(), Node::Row(_) | Node::Atom { .. }));
    }

    #[test]
    fn bar_with_no_spacing_glyph_still_parses() {
        // \bar emits U+203E (overline) which has no standalone glyph; the accent
        // path must remap to the combining macron and never hit atom_node.
        let ir = to_ir(r"\bar{y}", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        assert!(matches!(items[0], Node::Accent { .. }));
    }

    #[test]
    fn vec_parses_as_accent_not_superscript() {
        let ir = to_ir(r"\vec{B}", 16.0, Style::Text).unwrap();
        let Node::Row(items) = ir else { panic!() };
        assert!(
            matches!(items[0], Node::Accent { .. }),
            "\\vec must be an Accent, not a Subsup: got {:?}",
            items[0]
        );
    }

    #[test]
    fn lim_subscript_wraps_opname_base() {
        let ir = to_ir(r"\lim_{x}", 16.0, Style::Display).unwrap();
        let Node::Row(items) = ir else { panic!() };
        let Node::Subsup { base, sub: Some(_), .. } = &items[0] else {
            panic!("expected Subsup, got {:?}", items[0])
        };
        let Node::OpName { limits, .. } = base.as_ref() else {
            panic!("expected OpName base, got {:?}", base)
        };
        assert!(*limits);
    }
}
