use iced_math::MathRenderer;
use std::fs;

#[test]
fn svg_snapshots() {
    insta::glob!("corpus/*.tex", |path| {
        let raw = fs::read_to_string(path).unwrap();
        let src = raw.trim();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let display = src.contains(r"\sum")
            || src.contains(r"\int")
            || src.contains(r"\prod")
            || stem.contains("display");
        let bytes = MathRenderer::new()
            .display_style(display)
            .to_svg(src)
            .expect("parse must succeed for corpus");
        let s = String::from_utf8(bytes).expect("svg bytes must be utf-8");
        insta::assert_snapshot!(stem, s);
    });
}
