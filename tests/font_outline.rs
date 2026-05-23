use iced_math::font;

#[test]
fn outlines_capital_e_to_path_string() {
    let id = font::glyph_id('E').unwrap();
    let path = font::outline_path(id);
    assert!(
        path.contains('M'),
        "path should contain at least one Move: {}",
        path
    );
    assert!(
        path.contains('L') || path.contains('C') || path.contains('Q'),
        "path should contain at least one line/curve segment: {}",
        path
    );
    assert!(
        path.ends_with('Z') || path.contains("Z "),
        "path should be closed: {}",
        path
    );
}

#[test]
fn outline_uses_design_units_no_scaling() {
    let id = font::glyph_id('E').unwrap();
    let path = font::outline_path(id);
    let any_large = path
        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .filter_map(|s| s.parse::<f32>().ok())
        .any(|n| n.abs() > 50.0);
    assert!(any_large, "expected design-unit magnitudes, got: {}", path);
}
