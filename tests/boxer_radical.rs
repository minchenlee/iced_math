use iced_math::{boxer, ir::Style, parse};

#[test]
fn sqrt_height_exceeds_body_height() {
    let x = boxer::layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
    let s = boxer::layout(
        &parse::to_ir(r"\sqrt{x}", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    assert!(s.height > x.height);
}
