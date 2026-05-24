//! Render LaTeX to standalone SVG files using the Iced-free `MathRenderer` API.
//!
//! Run with:  cargo run --example to_svg
//! Writes .svg files into ./to_svg_out/ and prints one full SVG to stdout.

use iced_math::{Color, MathRenderer};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // (filename stem, LaTeX source, display-style?, fill color)
    let cases: &[(&str, &str, bool, Color)] = &[
        ("emc2", "E = mc^2", false, Color::BLACK),
        ("fraction", r"\frac{a + b}{c}", false, Color::BLACK),
        ("radical", r"\sqrt{x^2 + y^2}", false, Color::BLACK),
        ("sum", r"\sum_{i=1}^{n} i^2", true, Color::BLACK),
        ("colored", r"\frac{1}{2}", false, Color::rgb(0x22, 0x88, 0xff)),
    ];

    let out_dir = Path::new("to_svg_out");
    std::fs::create_dir_all(out_dir)?;

    for (stem, src, display, color) in cases {
        let svg = MathRenderer::new()
            .display_style(*display)
            .color(*color)
            .to_svg(src)?;
        let path = out_dir.join(format!("{stem}.svg"));
        std::fs::write(&path, &svg)?;
        println!("{src}  ->  {}  ({} bytes)", path.display(), svg.len());
    }

    // Show a complete SVG inline so the raw output is visible without opening a file.
    let raw = MathRenderer::new().to_svg(r"\frac{a}{b}")?;
    println!("\n--- raw SVG for \\frac{{a}}{{b}} ---\n{}", String::from_utf8(raw)?);

    Ok(())
}
