use iced_math::{boxer, ir::Style, parse};

#[test]
fn xsup2_taller_than_x() {
    let x = boxer::layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
    let xs = boxer::layout(&parse::to_ir("x^2", 16.0, Style::Text).unwrap(), Style::Text);
    assert!(xs.height > x.height);
}

#[test]
fn xsub_has_more_depth_than_x() {
    let x = boxer::layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
    let xs = boxer::layout(&parse::to_ir("x_i", 16.0, Style::Text).unwrap(), Style::Text);
    assert!(xs.depth > x.depth);
}
