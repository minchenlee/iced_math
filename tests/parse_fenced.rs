use iced_math::{
    ir::{Node, Style},
    parse,
};

#[test]
fn parses_left_right_paren() {
    let ir = parse::to_ir(r"\left( x \right)", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    assert_eq!(items.len(), 1);
    let Node::Fenced { open: _, close: _, body } = &items[0] else {
        panic!("expected Fenced")
    };
    assert!(matches!(body.as_ref(), Node::Row(_)));
}

#[test]
fn parses_left_right_brackets() {
    let ir = parse::to_ir(r"\left[ \frac{a}{b} \right]", 16.0, Style::Display).unwrap();
    let Node::Row(items) = ir else { panic!() };
    let Node::Fenced { .. } = &items[0] else {
        panic!("expected Fenced")
    };
}

#[test]
fn parses_left_dot_null_delim() {
    // `\left.` produces None for the open delimiter — should fall back to GlyphId(0)
    // and still yield a Fenced node.
    let ir = parse::to_ir(r"\left. x \right)", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    let Node::Fenced { open, .. } = &items[0] else {
        panic!("expected Fenced")
    };
    assert_eq!(open.0, 0);
}
