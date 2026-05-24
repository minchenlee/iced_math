# iced_math

Native LaTeX math widget for [Iced](https://iced.rs) 0.14. Pure Rust, **zero JavaScript runtime**.

## Status

Pre-1.0 — API may change. v0.2 supports a Tier 1 LaTeX subset (~50 commands).

## Installation

```toml
[dependencies]
iced_math = "0.2"
iced = { version = "0.14", features = ["wgpu", "tiny-skia"] }
```

## Quickstart

```rust
use iced::widget::column;
use iced::Element;

#[derive(Debug, Clone)]
enum Message {}

fn view(_state: &()) -> Element<'_, Message> {
    column![
        // Inline (text-style): sits on a text baseline.
        iced_math::inline("a^2 + b^2 = c^2"),
        // Block (display style): centered, larger operators.
        iced_math::block(r"\int_0^1 x^2 \, dx = \frac{1}{3}"),
    ]
    .into()
}
```

See `examples/viewer.rs` for a full demo:

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

## Supported LaTeX (v0.1)

- Atoms: letters, digits, basic operators (`+`, `−`, `×`, `÷`, `=`, `<`, `>`, `≤`, `≥`)
- Greek: `\alpha` … `\omega`, `\Gamma` … `\Omega`
- Super/subscripts: `x^2`, `a_i`, `x^{n+1}`, `a_i^j`
- Fractions: `\frac{a}{b}` (and `\dfrac`, `\tfrac`)
- Square roots: `\sqrt{x}`, `\sqrt[n]{x}`
- Large operators with limits: `\sum`, `\int`, `\prod` (display style places limits above/below)
- Delimiters: `\left( … \right)`, `\left[ … \right]` with auto-sizing variants

## Not yet supported (deferred to later releases)

- Matrices, `aligned`, `cases`, multiline environments — v0.2
- Accents (`\hat`, `\bar`, `\vec`, `\tilde`) — currently render misplaced (e.g. `\vec{B}` shows the arrow as a superscript); proper over-accent stacking is v0.2
- Binomials (`\binom{n}{k}`) — currently drawn with a fraction rule; the zero-thickness (open) binomial bar is v0.2
- Named limit-operators (`\lim`, `\max`, `\min`, `\sup`, `\inf`) — subscript renders to the right instead of stacked underneath; v0.2
- Multi-letter function names (`\sin`, `\cos`, `\log`, …) — currently typeset as spaced italic ordinaries instead of an upright, tightly-set operator; v0.2
- Unary vs. binary `-`/`+` disambiguation — a leading unary minus gets binary spacing; v0.2
- `GlyphAssembly` for extra-tall delimiters — v0.2
- AMS alphabets (`\mathbb`, `\mathfrak`, `\mathcal`) — fall back to the regular glyph; v0.3
- Sizing modes (`\scriptstyle`, `\displaystyle` overrides) — v0.3

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
