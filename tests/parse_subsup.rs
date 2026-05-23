use iced_math::{parse, ir::{Node, Style}};

#[test]
fn parses_superscript() {
    let ir = parse::to_ir("x^2", 16.0, Style::Text).unwrap();
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
    let ir = parse::to_ir("a_i", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    let Node::Subsup { sub, sup, .. } = &items[0] else { panic!() };
    assert!(sub.is_some());
    assert!(sup.is_none());
}

#[test]
fn parses_both_sub_and_sup() {
    let ir = parse::to_ir("a_i^j", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    let Node::Subsup { sub, sup, .. } = &items[0] else { panic!() };
    assert!(sub.is_some() && sup.is_some());
}

#[test]
fn parses_braced_exponent() {
    let ir = parse::to_ir("x^{n+1}", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    let Node::Subsup { sup: Some(sup), .. } = &items[0] else { panic!() };
    let Node::Row(inner) = sup.as_ref() else {
        panic!("expected Row inside exponent, got {:?}", sup)
    };
    assert_eq!(inner.len(), 3, "n + 1 = 3 atoms");
}
