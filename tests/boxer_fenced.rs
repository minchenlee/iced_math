use iced_math::{boxer, ir::Style, parse};

#[test]
fn fenced_around_frac_uses_taller_paren_variant_than_inline_paren() {
    let inline = boxer::layout(
        &parse::to_ir("(x)", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    let fenced = boxer::layout(
        &parse::to_ir(r"\left(\frac{1}{2}\right)", 16.0, Style::Text).unwrap(),
        Style::Text,
    );
    assert!(
        fenced.height > inline.height,
        "left/right should size to body; got fenced.height={} inline.height={}",
        fenced.height,
        inline.height
    );
}
