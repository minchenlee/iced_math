# iced_math

Native LaTeX math widget for [Iced](https://iced.rs) 0.14. Pure Rust, zero JavaScript runtime.

## Status
Pre-1.0 — API may change. v0.1 supports a Tier 1 subset (~50 commands: atoms, frac, sub/sup, sqrt, basic operators, parens, greek). Roadmap targets KaTeX-equivalent coverage at v1.0.

## Usage

```rust
use iced_math;

fn view(&self) -> Element<Message> {
    iced_math::block(r"E = mc^2")
}
```

## License
MIT OR Apache-2.0. Bundled font: see `assets/FONT-LICENSE.txt`.
