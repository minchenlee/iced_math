use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iced_math::{boxer, ir::Style, parse, svg};

fn bench_parse_layout(c: &mut Criterion) {
    c.bench_function("parse+layout emc2", |b| {
        b.iter(|| {
            let ir = parse::to_ir(black_box("E = mc^2"), 16.0, Style::Text).unwrap();
            black_box(boxer::layout(&ir, Style::Text));
        });
    });

    c.bench_function("parse+layout sum_display", |b| {
        b.iter(|| {
            let ir = parse::to_ir(black_box(r"\sum_{i=1}^{n} i^2"), 16.0, Style::Display).unwrap();
            black_box(boxer::layout(&ir, Style::Display));
        });
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    c.bench_function("parse+layout+emit emc2", |b| {
        b.iter(|| {
            let ir = parse::to_ir(black_box("E = mc^2"), 16.0, Style::Text).unwrap();
            let bx = boxer::layout(&ir, Style::Text);
            black_box(svg::emit(&bx));
        });
    });
}

criterion_group!(benches, bench_parse_layout, bench_full_pipeline);
criterion_main!(benches);
