use iced_math::font;

#[test]
fn integral_has_bigger_variant() {
    let id = font::glyph_id('∫').unwrap();
    // LMM's ∫ variant chain starts with the base glyph itself (~1112 du advance),
    // then a taller variant (~2223 du). Pick a target above the base advance so
    // we skip past the self-entry to a genuinely bigger glyph.
    let (variant, advance) = font::math_variant_vertical(id, 1500.0)
        .expect("integral must have a bigger vertical variant");
    assert!(
        variant != id,
        "should return a different glyph for bigger size"
    );
    assert!(advance >= 1500.0);
}

#[test]
fn returns_none_for_atom_without_variants() {
    let id = font::glyph_id('E').unwrap();
    assert!(font::math_variant_vertical(id, 50.0).is_none());
}
