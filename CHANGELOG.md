# Changelog

## [0.5.0] - 2026-05-30
### Added
- **Multiline display environments:** `align`, `align*`, `gather`, `gather*`, `equation`, `equation*`, `multline`, `alignat`, `alignedat`. Previously only the inner `aligned`/`split`/`cases`/`matrix`/`array` grids were recognized, so these document-level environments fell through to the unknown-grouping path and their `&` column / `\\` row separators were silently dropped — collapsing every multi-line derivation onto a single line with no error. `align`/`alignat`/`alignedat` lay out as right/left aligned pairs (matching `aligned`); `gather`/`gathered`/`equation`/`multline` as a single centered column.

## [0.4.0] - 2026-05-24
### Added
- **Math alphabets:** `\mathbb`, `\mathcal`, `\mathfrak`, `\mathbf`, `\mathsf`, `\mathtt`, `\mathit` (and the bold/sans/italic combinations pulldown-latex distinguishes). Letters and digits remap to their Unicode math-alphanumeric codepoints (e.g. `\mathbb{R}` → ℝ, `\mathcal{L}` → ℒ). The font state applies across a braced group and is correctly scoped — it does not leak past the group and resets across matrix cell/row boundaries. A character with no styled glyph in the bundled font falls back to its plain form.
- **Binomials:** `\binom{n}{k}` (and `\dbinom`, `\tbinom`) render as a ruleless stack inside auto-sized parentheses, reusing the fraction layout's axis spacing minus the rule.

### Changed
- README "supported LaTeX" list updated; version labels bumped to v0.4.

## [0.3.0] - 2026-05-24
### Added
- **Matrices and tabular environments:** `matrix`, `pmatrix`, `bmatrix`, `vmatrix`, `Bmatrix`, `Vmatrix`, plus `cases`, `aligned`, and `array`. Per-column alignment, per-row ascent/descent, and the whole grid centered on the math axis so it lines up with surrounding relations. Enclosing delimiters (e.g. `pmatrix`'s parens) auto-size to the grid.
- **Named operators:** `\sin`, `\cos`, `\log`, … render upright and tightly set (no inter-letter spacing) while spacing as a single operator against their argument. Limit operators (`\lim`, `\max`, `\min`, `\sup`, `\inf`, `\det`, `\gcd`, …) stack their subscript underneath in display style.
- **Accents:** `\hat`, `\bar`, `\vec`, `\tilde`, `\dot`, `\ddot`, `\check`, `\breve`, `\acute`, `\grave`, centered horizontally over the body and raised above it (using the font's combining-accent glyphs and the MATH `AccentBaseHeight`).

### Fixed
- Function names (`\sin` etc.) no longer typeset as spaced italic ordinaries; `\bar` and other spacing-glyph accents no longer error or render as a raised superscript.

### Changed
- README "supported LaTeX" list updated; the Iced quickstart now compiles (binds `inline`/`block` to a typed `Element` and documents the turbofish fallback) and the install snippet enables iced's `svg` feature.

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
