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

#[test]
fn sqrt_with_long_degree_widens_box() {
    // Regression: previously layout_radical placed the degree at x=0 but reported
    // width = surd + body, so a wide degree (e.g. `\sqrt[12345]{x}`) extended
    // past the parent box's bounds and clipped in the SVG viewport.
    let s_normal = boxer::layout(
        &parse::to_ir(r"\sqrt{x}", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    let s_long = boxer::layout(
        &parse::to_ir(r"\sqrt[12345]{x}", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    assert!(
        s_long.width > s_normal.width + 10.0,
        "wide degree must widen the parent box: long={} normal={}",
        s_long.width,
        s_normal.width,
    );
}
