use iced_math::font;

#[test]
fn metrics_for_capital_e() {
    let id = font::glyph_id('E').unwrap();
    let m = font::glyph_metrics(id, 16.0);
    assert!(m.advance > 0.0);
    assert!(m.height > 0.0);
    assert!(m.depth >= 0.0);
    assert!(m.height > m.depth);
}

#[test]
fn metrics_scale_linearly_with_size() {
    let id = font::glyph_id('E').unwrap();
    let m1 = font::glyph_metrics(id, 10.0);
    let m2 = font::glyph_metrics(id, 20.0);
    let ratio = m2.advance / m1.advance;
    assert!((ratio - 2.0).abs() < 1e-3, "expected 2.0 ratio, got {}", ratio);
}
