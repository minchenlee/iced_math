# iced_math

Native LaTeX math widget for [Iced](https://iced.rs) 0.14. Pure Rust, **zero JavaScript runtime**.

## Status

Pre-1.0 — API may change. v0.1 supports a Tier 1 LaTeX subset (~50 commands).

## Installation

```toml
[dependencies]
iced_math = "0.1"
iced = { version = "0.14", features = ["svg", "advanced"] }
```

## Quickstart

```rust
use iced::Element;
use iced_math;

fn view(&self) -> Element<Message> {
    // Inline math (text-style)
    iced_math::inline(r"E = mc^2")

    // Or block (display style, centered)
    // iced_math::block(r"\sum_{i=1}^{n} i = \frac{n(n+1)}{2}")
}
```

See `examples/viewer.rs` for a full demo:

```bash
cargo run --example viewer
```

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
- Accents (`\hat`, `\bar`, `\tilde`) — v0.2
- `GlyphAssembly` for extra-tall delimiters — v0.2
- AMS symbols (`\mathbb`, `\mathfrak`, `\mathcal`) — v0.3
- Color, sizing modes — v0.3

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
