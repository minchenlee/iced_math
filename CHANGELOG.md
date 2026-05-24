# Changelog

## [0.2.0] - 2026-05-24
### Added
- `MathRenderer` — a low-level, Iced-free builder (`new`, `font_size`, `display_style`, `color`, `to_svg`) that renders LaTeX to standalone SVG bytes for server-side rendering, export, docs, or tests.
- Public `Color` type (`Color::BLACK`, `Color::rgb`) for setting the glyph fill.
- Public `Error` type (`Parse`, `InvalidFontSize`) implementing `Display`, `std::error::Error`, and `PartialEq`. `MathRenderer::to_svg` validates the font size (must be finite and strictly positive) before rendering.
- Glyph color support: non-default colors wrap the body in a single inheriting `<g fill>` group; the default (black) output is byte-for-byte unchanged.

### Changed
- **Breaking:** the internal modules `boxer`, `font`, `ir`, `parse`, `spacing`, and `svg` are now private. The supported public API for the 0.x series is `inline`, `block`, `MathRenderer`, `Color`, and `Error`.
- **Breaking:** the library no longer enables iced's `x11` feature. It keeps `thread-pool` (iced requires an executor at compile time) but leaves Linux windowing to consumers; the `viewer` example re-adds `x11` as a dev-dependency.
- Expanded README (full Iced view example, raw-SVG `to_svg` example, stable-API note) and added rustdoc examples.

## [0.1.1] - 2026-05-24
### Fixed
- Fraction rule now stays visible on narrow content (e.g. `\frac{1}{2}`) via a small horizontal overhang.
- Fraction rule is vertically centred between the numerator's and denominator's ink, instead of floating a bare-digit numerator high above a denominator pinned close to the bar.
- Tall fraction content (`\frac{\sqrt{\pi}}{2}`) no longer crosses the rule — numerator depth / denominator height now enforce the OpenType MATH `Fraction*GapMin` clearance.
- Radical: the surd glyph's top is aligned to the vinculum so the hook, stem, and overline form one continuous stroke (previously a detached/misaligned connector).
- SVG viewport is padded by 1px so edge glyphs (e.g. fraction denominators with near-zero depth) no longer lose their bottom row to rasterizer antialiasing.

### Changed
- `viewer` example now activates `iced`'s `wgpu` + `tiny-skia` renderer features (dev-dependency only); the library itself remains renderer-agnostic.

## [0.1.0] - 2026-05-23
### Added
- Initial release.
- Tier 1 LaTeX subset (~50 commands):
  - Atoms (letters, digits, basic operators) with TeX 8×8 inter-atom spacing.
  - Greek letters (lowercase + uppercase).
  - Super/subscripts (`x^2`, `a_i`, `x^{n+1}`, `a_i^j`).
  - Fractions (`\frac{a}{b}`).
  - Square roots (`\sqrt{x}`, `\sqrt[n]{x}`).
  - Large operators (`\sum`, `\int`, `\prod`) with display-mode limits.
  - Delimiters (`\left ( \right )`, `\left [ \right ]`) with vertical glyph variants.
- Pure-Rust pipeline: `pulldown-latex` → IR → TeX-style boxer (uses OpenType MATH table via `ttf-parser`) → SVG bytes → `iced::widget::svg`.
- Bundled Latin Modern Math font (~717KB) under GFL license.
- Two public functions: `iced_math::inline()` and `iced_math::block()`.
- Error fallback: failed parse renders source as red monospace text.
- 50 tests across 24 suites (parse, boxer, SVG, snapshots, pixel).
- 30-entry corpus for SVG + pixel regression via `insta` + `resvg` + `tiny-skia`.
- Criterion benches for parse+layout pipeline.
- GitHub Actions CI matrix: linux/macos/windows × stable/MSRV-1.75.
