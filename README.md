# iced_math

Native LaTeX math widget for [Iced](https://iced.rs) 0.14. Pure Rust, **zero JavaScript runtime**.

## Status

Pre-1.0 — API may change. v0.2 supports a Tier 1 LaTeX subset (~50 commands).

## Installation

```toml
[dependencies]
iced_math = "0.2"
# iced_math renders through iced's `svg` widget, so your iced dependency must
# enable a renderer (`wgpu` and/or `tiny-skia`) plus the `svg` feature.
iced = { version = "0.14", features = ["wgpu", "tiny-skia", "svg"] }
```

## Quickstart

`inline` and `block` are generic over `Message`, `Theme`, and `Renderer`. The
compiler infers those from the surrounding `Element` type, so bind each call to a
typed `Element` (or annotate the `let`) — a bare call inside `column![ … ]` has
nothing to infer from and will fail with a "type annotations needed" error.

```rust
use iced::widget::column;
use iced::Element;

#[derive(Debug, Clone)]
enum Message {}

fn view(_state: &()) -> Element<'_, Message> {
    // Inline (text-style): sits on a text baseline.
    let inline: Element<'_, Message> = iced_math::inline("a^2 + b^2 = c^2");
    // Block (display style): centered, larger operators.
    let block: Element<'_, Message> = iced_math::block(r"\int_0^1 x^2 \, dx = \frac{1}{3}");

    column![inline, block].into()
}
```

If you can't bind to a typed `Element`, use the turbofish instead:

```rust,ignore
iced_math::inline::<Message, iced::Theme, iced::Renderer>("a^2 + b^2 = c^2")
```

A full runnable demo lives in `examples/viewer.rs` (the example enables iced's
`wgpu`/`tiny-skia` renderer as a dev-dependency):

```bash
cargo run --example viewer
```

## Without Iced (raw SVG)

The `MathRenderer` builder produces standalone SVG bytes — useful for docs,
export tools, server rendering, or tests — with no Iced dependency at the call site:

```rust
let svg: Vec<u8> = iced_math::MathRenderer::new()
    .font_size(24.0)
    .display_style(true)
    .color(iced_math::Color::rgb(0x22, 0x22, 0x22))
    .to_svg(r"\sqrt{x^2 + y^2}")
    .expect("valid LaTeX");
std::fs::write("out.svg", svg).unwrap();
```

## Stable API

For the `0.x` series the supported public API is: `inline`, `block`,
`MathRenderer`, `Color`, and `Error`. Everything else is internal and may change
between minor versions.

## Supported LaTeX (v0.2)

- Atoms: letters, digits, basic operators (`+`, `−`, `×`, `÷`, `=`, `<`, `>`, `≤`, `≥`)
- Greek: `\alpha` … `\omega`, `\Gamma` … `\Omega`
- Super/subscripts: `x^2`, `a_i`, `x^{n+1}`, `a_i^j`
- Fractions: `\frac{a}{b}` (and `\dfrac`, `\tfrac`)
- Square roots: `\sqrt{x}`, `\sqrt[n]{x}`
- Large operators with limits: `\sum`, `\int`, `\prod` (display style places limits above/below)
- Named operators: `\sin`, `\cos`, `\log`, … (upright, tight-set) and limit operators `\lim`, `\max`, `\min`, `\sup`, `\inf`, … (subscript stacks underneath in display style)
- Accents: `\hat`, `\bar`, `\vec`, `\tilde`, `\dot`, `\ddot`, `\check`, `\breve`, `\acute`, `\grave` (centered over the body)
- Matrices: `matrix`, `pmatrix`, `bmatrix`, `vmatrix`, `Bmatrix`, `Vmatrix`, plus `cases`, `aligned`, and `array` (per-column alignment, axis-centered)
- Delimiters: `\left( … \right)`, `\left[ … \right]` with auto-sizing variants

## Not yet supported (deferred to later releases)

Roughly ordered by planned priority:

- Multi-line display environments (`align`, `gather`, `multline`) and `\hline`/column separators in `array`
- `GlyphAssembly` for extra-tall delimiters — `\left( … \right)` won't grow past the largest single glyph
- Binomials (`\binom{n}{k}`) — currently drawn with a fraction rule; should be ruleless
- Unary vs. binary `-`/`+` disambiguation — a leading unary minus gets binary spacing
- AMS alphabets (`\mathbb`, `\mathfrak`, `\mathcal`) — fall back to the regular glyph
- Sizing modes (`\scriptstyle`, `\displaystyle` overrides)

## How it works

```
LaTeX source
    ↓ pulldown-latex
event stream
    ↓ parse::to_ir
IR (Node tree)
    ↓ boxer::layout
positioned Box tree (TeX-style boxes via OpenType MATH table)
    ↓ svg::emit
SVG bytes
    ↓ iced::widget::svg
Element<Message, Theme, Renderer>
```

Font: bundled [Latin Modern Math](https://www.gust.org.pl/projects/e-foundry/lm-math) (GFL license).

## Error handling

Parse failures render the source as red monospace text — never panic, always render something.

## License

Code: MIT OR Apache-2.0. Font: GUST Font License (see `assets/FONT-LICENSE.txt`).
