use iced_math::{parse, ir::{Node, AtomClass}};

#[test]
fn parses_single_letter() {
    let ir = parse::to_ir("x", 16.0, iced_math::ir::Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!("expected Row, got {:?}", ir) };
    assert_eq!(items.len(), 1);
    let Node::Atom { class, .. } = &items[0] else { panic!() };
    assert_eq!(*class, AtomClass::Ord);
}

#[test]
fn parses_two_letters_as_row() {
    let ir = parse::to_ir("xy", 16.0, iced_math::ir::Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    assert_eq!(items.len(), 2);
}

#[test]
fn classifies_plus_as_bin() {
    let ir = parse::to_ir("a+b", 16.0, iced_math::ir::Style::Text).unwrap();
    let Node::Row(items) = ir else { panic!() };
    let Node::Atom { class: c2, .. } = &items[1] else { panic!() };
    assert_eq!(*c2, AtomClass::Bin);
}
