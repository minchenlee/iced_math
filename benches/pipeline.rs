use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iced_math::MathRenderer;

fn bench_full_pipeline(c: &mut Criterion) {
    c.bench_function("to_svg emc2 (text)", |b| {
        let r = MathRenderer::new();
        b.iter(|| {
            black_box(r.to_svg(black_box("E = mc^2")).unwrap());
        });
    });

    c.bench_function("to_svg sum (display)", |b| {
        let r = MathRenderer::new().display_style(true);
        b.iter(|| {
            black_box(r.to_svg(black_box(r"\sum_{i=1}^{n} i^2")).unwrap());
        });
    });
}

criterion_group!(benches, bench_full_pipeline);
criterion_main!(benches);
