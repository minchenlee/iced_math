use iced_math::{boxer, ir::Style, parse};

#[test]
fn sum_display_has_limits_above_below() {
    let bare = boxer::layout(
        &parse::to_ir(r"\sum", 16.0, Style::Display).unwrap(),
        Style::Display,
    );
    let s = boxer::layout(
        &parse::to_ir(r"\sum_{i=1}^{n}", 16.0, Style::Display).unwrap(),
        Style::Display,
    );
    assert!(
        s.height > bare.height,
        "display sum with sup limit should be taller than bare sum: bare.h={} sum.h={}",
        bare.height,
        s.height,
    );
    assert!(
        s.depth > bare.depth,
        "display sum with sub limit should have more depth than bare sum: bare.d={} sum.d={}",
        bare.depth,
        s.depth,
    );
}
