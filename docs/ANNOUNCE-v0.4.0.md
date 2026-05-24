# Announce kit — iced_math v0.4.0

Ready-to-use copy for the v0.4.0 release. **Nothing here is posted automatically** —
publish the crate first (`cargo publish`), then use whichever of these you want.

Note: v0.3.0 was tagged but never published to crates.io, so v0.4.0 is the first
public release to carry matrices, named operators, accents, math alphabets, and
binomials. The copy below covers the whole set.

---

## GitHub release body (tag `v0.4.0`)

> ## iced_math v0.4.0
>
> Native, zero-JS LaTeX math for [Iced](https://iced.rs) 0.14 — now expressive
> enough for real STEM and ML content.
>
> ### New in 0.4
> - **Math alphabets** — `\mathbb`, `\mathcal`, `\mathfrak`, `\mathbf`, `\mathsf`,
>   `\mathtt`, `\mathit`. `\mathbb{R}` → ℝ, `\mathcal{L}` → ℒ, `\mathbb{E}` → 𝔼, …
>   (Unicode math-alphanumeric remapping with a plain-glyph fallback).
> - **Binomials** — `\binom{n}{k}` (and `\dbinom`, `\tbinom`): a ruleless stack
>   in auto-sized parentheses.
>
> ### Also included (first public release of these)
> - **Matrices & tabular environments** — `matrix`, `pmatrix`, `bmatrix`,
>   `vmatrix`, `Bmatrix`, `Vmatrix`, plus `cases`, `aligned`, and `array`.
>   Per-column alignment, axis-centered; enclosing delimiters auto-size.
> - **Named operators** — `\sin`, `\cos`, `\log`, … upright and tight; limit
>   operators (`\lim`, `\max`, `\sup`, …) stack their subscript underneath in
>   display style.
> - **Accents** — `\hat`, `\bar`, `\vec`, `\tilde`, `\dot`, `\ddot`, `\check`,
>   `\breve`, `\acute`, `\grave`, centered over the body.
>
> Full notes: [CHANGELOG.md](CHANGELOG.md)

---

## Iced Discord (#show-and-tell or similar)

> Pushed **iced_math 0.4.0** — a pure-Rust, zero-JS LaTeX math widget for Iced 0.14.
> It renders LaTeX → SVG via the OpenType MATH table and drops straight into an
> `Element`. This release covers the pieces real math needs: **matrices**
> (`pmatrix`/`cases`/`aligned`/…), **named operators** (`\sin`, `\lim` with limit
> stacking), **accents** (`\vec`, `\hat`, …), **math alphabets** (`\mathbb{R}`,
> `\mathcal{L}`, `\mathbb{E}`), and **binomials** (`\binom`). No webview, no JS.
> `cargo add iced_math`. Feedback welcome 🙂
> https://crates.io/crates/iced_math

---

## r/rust

**Title:** iced_math 0.4.0 — native zero-JS LaTeX math widget for Iced (matrices, \mathbb, \binom)

> iced_math renders LaTeX math natively in [Iced](https://iced.rs) 0.14 apps —
> pure Rust, no webview, no JavaScript engine. It parses LaTeX, lays it out with
> the bundled font's OpenType MATH table (TeX-style boxing), emits SVG, and hands
> you an Iced `Element`.
>
> What it handles now:
> - **Matrices / cases / aligned / array** with per-column alignment
> - **Named operators** (`\sin`, `\cos`, `\lim` with limit stacking)
> - **Accents** (`\vec`, `\hat`, `\bar`, `\tilde`, …)
> - **Math alphabets** (`\mathbb{R}`, `\mathcal{L}`, `\mathbb{E}` — handy for ML/stats)
> - **Binomials** (`\binom{n}{k}`)
>
> There's also a raw, Iced-free `MathRenderer::to_svg(...)` if you just want SVG
> bytes for export/docs/server rendering.
>
> Crate: https://crates.io/crates/iced_math · Repo: https://github.com/minchenlee/iced_math
>
> Still pre-1.0 — known gaps (multiline `align`/`gather`, tall delimiter assembly,
> sizing modes) are listed in the README.
