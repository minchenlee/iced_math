use iced_math::MathRenderer;
use std::fs;

fn rasterize_svg(svg_bytes: &[u8], scale: f32) -> Vec<u8> {
    let opts = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(svg_bytes, &opts).expect("usvg parse");
    let size = tree.size();
    let w = (size.width() * scale).ceil().max(1.0) as u32;
    let h = (size.height() * scale).ceil().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(w, h).expect("pixmap alloc");
    let ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, ts, &mut pixmap.as_mut());
    pixmap.encode_png().expect("png encode")
}

#[test]
fn pixel_snapshots() {
    insta::glob!("corpus/*.tex", |path| {
        let raw = fs::read_to_string(path).unwrap();
        let src = raw.trim();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let display = src.contains(r"\sum")
            || src.contains(r"\int")
            || src.contains(r"\prod")
            || stem.contains("display");
        let svg_bytes = MathRenderer::new()
            .display_style(display)
            .to_svg(src)
            .expect("parse must succeed for corpus");
        let png = rasterize_svg(&svg_bytes, 4.0);
        insta::assert_binary_snapshot!(&format!("{stem}.png")[..], png);
    });
}
