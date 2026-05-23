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

#[test]
fn returns_largest_when_target_exceeds_all_variants() {
    // Regression: previously math_variant_vertical returned None when no
    // variant reached the target, causing callers' `unwrap_or(base)` to fall
    // all the way back to the base glyph — even though the font ships much
    // larger variants. Now we fall back to the largest available variant.
    let id = font::glyph_id('∫').unwrap();
    let (variant, h) = font::math_variant_vertical(id, 1e9)
        .expect("should fall back to largest variant, not None");
    assert!(
        variant != id,
        "should return a non-base variant; got base glyph"
    );
    assert!(
        h > 1500.0,
        "largest variant should be substantially bigger than base; got {}",
        h,
    );
}
