# Announce kit — iced_math v0.3.0

Ready-to-use copy for the v0.3.0 release. **Nothing here is posted automatically** —
publish the crate first (`cargo publish`), then use whichever of these you want.

---

## GitHub release body (tag `v0.3.0`)

> ## iced_math v0.3.0
>
> Native, zero-JS LaTeX math for [Iced](https://iced.rs) 0.14 — now with the
> three things that were blocking real STEM content.
>
> ### New
> - **Matrices & tabular environments** — `matrix`, `pmatrix`, `bmatrix`,
>   `vmatrix`, `Bmatrix`, `Vmatrix`, plus `cases`, `aligned`, and `array`.
>   Per-column alignment, axis-centered so they line up with surrounding
>   relations; enclosing delimiters auto-size to the grid.
> - **Named operators** — `\sin`, `\cos`, `\log`, … render upright and tight;
>   limit operators (`\lim`, `\max`, `\sup`, …) stack their subscript underneath
>   in display style.
> - **Accents** — `\hat`, `\bar`, `\vec`, `\tilde`, `\dot`, `\ddot`, `\check`,
>   `\breve`, `\acute`, `\grave`, centered over the body.
>
> ### Fixed
> - Function names no longer typeset as spaced italics; `\bar` and other
>   spacing-glyph accents no longer error or render as a raised superscript.
> - The README Iced quickstart now compiles (binds to a typed `Element`; the
>   install snippet enables iced's `svg` feature).
>
> Full notes: [CHANGELOG.md](CHANGELOG.md)

---

## Iced Discord (#show-and-tell or similar)

> Pushed **iced_math 0.3.0** — a pure-Rust, zero-JS LaTeX math widget for Iced 0.14.
> This release adds the big missing pieces: **matrices** (`pmatrix`/`bmatrix`/
> `cases`/`aligned`/…), **named operators** (`\sin`, `\lim` with proper limit
> stacking), and **accents** (`\vec`, `\hat`, `\bar`, …). No webview, no JS — it
> renders LaTeX → SVG via the OpenType MATH table and drops straight into an
> `Element`. `cargo add iced_math`. Feedback welcome 🙂
> https://crates.io/crates/iced_math

---

## r/rust

**Title:** iced_math 0.3.0 — native zero-JS LaTeX math widget for Iced (now with matrices)

> iced_math renders LaTeX math natively in [Iced](https://iced.rs) 0.14 apps —
> pure Rust, no webview, no JavaScript engine. It parses LaTeX, lays it out with
> the bundled font's OpenType MATH table (TeX-style boxing), emits SVG, and hands
> you an Iced `Element`.
>
> 0.3.0 closes the gaps that mattered most for real math:
> - **Matrices / cases / aligned / array** with per-column alignment
> - **Named operators** (`\sin`, `\cos`, `\lim` with limit stacking)
> - **Accents** (`\vec`, `\hat`, `\bar`, `\tilde`, …)
>
> There's also a raw, Iced-free `MathRenderer::to_svg(...)` if you just want SVG
> bytes for export/docs/server rendering.
>
> Crate: https://crates.io/crates/iced_math · Repo: https://github.com/minchenlee/iced_math
>
> Still pre-1.0 — known gaps (multiline `align`/`gather`, `\binom`, blackboard/
> calligraphic alphabets, tall delimiter assembly) are listed in the README.
