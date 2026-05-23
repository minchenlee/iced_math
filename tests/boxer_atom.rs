use iced_math::{boxer, ir::Style, parse};

#[test]
fn single_atom_box_has_expected_width() {
    let ir = parse::to_ir("x", 16.0, Style::Text).unwrap();
    let b = boxer::layout(&ir, Style::Text);
    assert!(b.width > 0.0, "x must have positive width");
    assert!(b.height > 0.0, "x must have positive height");
}

#[test]
fn ab_is_wider_than_a() {
    let a = boxer::layout(&parse::to_ir("a", 16.0, Style::Text).unwrap(), Style::Text);
    let ab = boxer::layout(&parse::to_ir("ab", 16.0, Style::Text).unwrap(), Style::Text);
    assert!(ab.width > a.width);
}

#[test]
fn aplusb_wider_than_ab_due_to_med_spacing() {
    let ab = boxer::layout(&parse::to_ir("ab", 16.0, Style::Text).unwrap(), Style::Text);
    let aplus = boxer::layout(&parse::to_ir("a+b", 16.0, Style::Text).unwrap(), Style::Text);
    assert!(
        aplus.width > ab.width,
        "a+b should be wider than ab due to Med spacing around +"
    );
}
