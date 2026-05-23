# Changelog

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
