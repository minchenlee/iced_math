use iced_math::{boxer, ir::Style, parse};

#[test]
fn frac_height_exceeds_num_height_alone() {
    let half = boxer::layout(
        &parse::to_ir(r"\frac{1}{2}", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    let one = boxer::layout(&parse::to_ir("1", 16.0, Style::Text).unwrap(), Style::Text);
    assert!(
        half.height > one.height,
        "frac should be taller than just the numerator"
    );
    assert!(
        half.depth > 0.0,
        "frac should have nonzero depth (denominator below axis)"
    );
}

#[test]
fn frac_width_at_least_max_of_num_den() {
    // Fraction children are laid out at sub-style (Script), so compare against
    // num/den rendered at Script style, not at the fraction's outer Text style.
    let f = boxer::layout(
        &parse::to_ir(r"\frac{abc}{de}", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    let num = boxer::layout(
        &parse::to_ir("abc", 16.0, Style::Text).unwrap(),
        Style::Script,
    );
    let den = boxer::layout(
        &parse::to_ir("de", 16.0, Style::Text).unwrap(),
        Style::Script,
    );
    assert!(
        f.width >= num.width.max(den.width),
        "frac width {} should be >= max(num_script {}, den_script {})",
        f.width,
        num.width,
        den.width,
    );
}

#[test]
fn frac_baseline_at_axis_sanity() {
    // For \frac{1}{2}, total ink height (height + depth) should exceed 1em (16px).
    let f = boxer::layout(
        &parse::to_ir(r"\frac{1}{2}", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    assert!(
        f.height + f.depth > 16.0,
        "frac total ink height should exceed 1em, got {} + {} = {}",
        f.height,
        f.depth,
        f.height + f.depth
    );
}

#[test]
fn row_baselines_align() {
    // Each child in an HBox should have offset.y == parent.height - child.height
    // so its baseline coincides with the parent's baseline.
    let ir = parse::to_ir("a", 16.0, Style::Text).unwrap();
    let b = boxer::layout(&ir, Style::Text);
    let boxer::BoxKind::HBox(children) = &b.kind else {
        panic!("expected HBox at row top, got {:?}", b.kind);
    };
    for c in children {
        let expected_y = b.height - c.child.height;
        assert!(
            (c.offset.y - expected_y).abs() < 0.01,
            "child top offset should equal parent.height - child.height; got {} expected {}",
            c.offset.y,
            expected_y
        );
    }
}
