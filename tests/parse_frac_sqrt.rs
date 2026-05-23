use iced_math::{
    ir::{Node, Style},
    parse,
};

#[test]
fn parses_frac() {
    let ir = parse::to_ir(r"\frac{1}{2}", 16.0, Style::Text).unwrap();
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
    let ir = parse::to_ir(r"\sqrt{x}", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    let Node::Radical { degree, body } = &items[0] else {
        panic!("expected Radical")
    };
    assert!(degree.is_none());
    assert!(matches!(body.as_ref(), Node::Row(_)));
}

#[test]
fn parses_sqrt_with_degree() {
    let ir = parse::to_ir(r"\sqrt[3]{x}", 16.0, Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    let Node::Radical {
        degree: Some(_),
        body: _,
    } = &items[0]
    else {
        panic!()
    };
}
