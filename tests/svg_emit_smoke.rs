use iced_math::{boxer, ir::Style, parse, svg};

#[test]
fn emits_well_formed_svg_for_atom() {
    let ir = parse::to_ir("x", 16.0, Style::Text).unwrap();
    let b = boxer::layout(&ir, Style::Text);
    let bytes = svg::emit(&b);
    let s = std::str::from_utf8(&bytes).unwrap();
    assert!(s.starts_with("<svg"));
    assert!(s.ends_with("</svg>"));
    assert!(s.contains("<path"), "atom should emit at least one path");
}

#[test]
fn emits_rect_for_frac_rule() {
    let ir = parse::to_ir(r"\frac{1}{2}", 16.0, Style::Text).unwrap();
    let b = boxer::layout(&ir, Style::Text);
    let bytes = svg::emit(&b);
    let s = std::str::from_utf8(&bytes).unwrap();
    assert!(s.contains("<rect"), "frac should emit a rule rect");
}
