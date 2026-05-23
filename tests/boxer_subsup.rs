use iced_math::{boxer, ir::Style, parse};

#[test]
fn script_glyph_size_smaller_than_base() {
    // Regression: prior to v0.1.1, parse stored style-scaled font_size on atoms,
    // so by the time layout_subsup passed `script_style` to `layout(sup, ...)`
    // the atom's stored size was unchanged. Now atoms store the *base* size and
    // the boxer applies style scaling, so `x^2`'s `2` must render at 0.7×base.
    let bx = boxer::layout(
        &parse::to_ir("x^2", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    // Top-level Row → HBox containing one Subsup → HBox([base x, sup 2]).
    let boxer::BoxKind::HBox(top_children) = &bx.kind else {
        panic!("expected HBox at top, got {:?}", bx.kind);
    };
    let subsup = &top_children
        .last()
        .expect("at least one child at top")
        .child;
    let boxer::BoxKind::HBox(inner) = &subsup.kind else {
        panic!("expected HBox (subsup), got {:?}", subsup.kind);
    };
    let sup_child = inner.last().expect("subsup should have at least one child");
    let boxer::BoxKind::Glyph { font_size, .. } = sup_child.child.kind else {
        panic!("expected glyph for sup, got {:?}", sup_child.child.kind);
    };
    assert!(
        font_size < 16.0,
        "sup font_size should be < base 16; got {}",
        font_size
    );
    assert!(
        (font_size - 16.0 * 0.7).abs() < 0.01,
        "expected ~11.2 (16 * 0.7), got {}",
        font_size
    );
}

#[test]
fn xsup2_taller_than_x() {
    let x = boxer::layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
    let xs = boxer::layout(
        &parse::to_ir("x^2", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    assert!(xs.height > x.height);
}

#[test]
fn xsub_has_more_depth_than_x() {
    let x = boxer::layout(&parse::to_ir("x", 16.0, Style::Text).unwrap(), Style::Text);
    let xs = boxer::layout(
        &parse::to_ir("x_i", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    assert!(xs.depth > x.depth);
}
