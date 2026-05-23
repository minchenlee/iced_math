use iced_math::ir::{AtomClass, Node, Style};
use iced_math::parse;

#[test]
fn parses_alpha() {
    let ir = parse::to_ir(r"\alpha", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    assert_eq!(items.len(), 1);
    let Node::Atom { class, .. } = &items[0] else {
        panic!()
    };
    assert_eq!(*class, AtomClass::Ord);
}

#[test]
fn parses_capital_gamma() {
    let ir = parse::to_ir(r"\Gamma", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    assert_eq!(items.len(), 1);
}

#[test]
fn parses_sum_with_limits_in_display() {
    let ir = parse::to_ir(r"\sum_{i=1}^{n}", 16.0, Style::Display).unwrap();
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
    let ir = parse::to_ir(r"\sum_{i=1}^{n}", 16.0, Style::Text).unwrap();
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
