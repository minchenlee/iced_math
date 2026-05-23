use iced_math::{boxer, ir::Style, parse, svg};
use std::fs;

#[test]
fn svg_snapshots() {
    insta::glob!("corpus/*.tex", |path| {
        let raw = fs::read_to_string(path).unwrap();
        let src = raw.trim();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let style = if src.contains(r"\sum")
            || src.contains(r"\int")
            || src.contains(r"\prod")
            || stem.contains("display")
        {
            Style::Display
        } else {
            Style::Text
        };
        let ir = parse::to_ir(src, 16.0, style).expect("parse must succeed for corpus");
        let b = boxer::layout(&ir, style);
        let bytes = svg::emit(&b);
        let s = String::from_utf8(bytes).expect("svg bytes must be utf-8");
        insta::assert_snapshot!(stem, s);
    });
}
