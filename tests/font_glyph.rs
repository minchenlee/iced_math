use iced_math::font;

#[test]
fn maps_ascii_letter_to_glyph_id() {
    let id = font::glyph_id('E').expect("E must exist in bundled math font");
    assert!(id.0 > 0);
}

#[test]
fn maps_greek_alpha() {
    let id = font::glyph_id('α').expect("α must exist");
    assert!(id.0 > 0);
}

#[test]
fn returns_none_for_unmapped_codepoint() {
    assert!(font::glyph_id('\u{E000}').is_none());
}
