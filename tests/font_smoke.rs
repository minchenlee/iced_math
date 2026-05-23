use iced_math::font;

#[test]
fn face_loads_and_has_math_table() {
    let upem = font::units_per_em();
    assert!(upem > 0.0, "units_per_em should be > 0, got {}", upem);
    assert!(font::has_math_table(), "bundled font must have MATH table");
}
