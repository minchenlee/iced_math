//! Native LaTeX math widget for Iced 0.14.
//!
//! Two entry points:
//! - [`inline`] for in-line math (text-style sizing).
//! - [`block`] for display math (centered, larger ops with limits above/below).
//!
//! Returns an [`iced::Element`] you can drop into any view tree.

pub(crate) static FONT_BYTES: &[u8] = include_bytes!("../assets/LatinModernMath.otf");

mod boxer;
mod font;
mod ir;
mod parse;
mod spacing;
mod svg;
mod widget;

mod color;
pub use color::Color;

mod error;
pub use error::Error;

use iced::advanced::svg::Renderer as SvgRenderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::widget::{container, svg as svg_widget, text};
use iced::{Element, Length};

/// Low-level math renderer. Configure size, display-style, and fill color,
/// then call [`MathRenderer::to_svg`] to produce raw SVG bytes — no Iced required.
///
/// Named `MathRenderer` (not `Renderer`) so it does not collide with
/// `iced::Renderer` or the generic `Renderer` type parameter on [`inline`]/[`block`].
///
/// ```
/// let bytes = iced_math::MathRenderer::new()
///     .font_size(24.0)
///     .display_style(true)
///     .color(iced_math::Color::rgb(0x33, 0x33, 0x33))
///     .to_svg(r"\frac{1}{2}")
///     .unwrap();
/// assert!(bytes.starts_with(b"<svg"));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct MathRenderer {
    font_size: f32,
    display: bool,
    color: Color,
}

impl MathRenderer {
    /// A renderer at 16 px, text-style (inline), black fill — same defaults as [`inline`].
    pub fn new() -> Self {
        MathRenderer {
            font_size: 16.0,
            display: false,
            color: Color::BLACK,
        }
    }

    /// Set the base font size in pixels. Stored as-is; validated in
    /// [`to_svg`](Self::to_svg) (must be finite and strictly positive).
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_size = px;
        self
    }

    /// `true` = display style (centered, large operators with limits);
    /// `false` = inline text style.
    pub fn display_style(mut self, yes: bool) -> Self {
        self.display = yes;
        self
    }

    /// Set the glyph fill color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Render `src` to standalone SVG bytes.
    ///
    /// # Errors
    /// - [`Error::InvalidFontSize`] if the configured font size is not finite
    ///   and strictly positive.
    /// - [`Error::Parse`] if the LaTeX cannot be parsed or contains an
    ///   unsupported construct.
    pub fn to_svg(&self, src: &str) -> Result<Vec<u8>, Error> {
        if !self.font_size.is_finite() || self.font_size <= 0.0 {
            return Err(Error::InvalidFontSize(self.font_size));
        }
        let style = if self.display {
            ir::Style::Display
        } else {
            ir::Style::Text
        };
        let node = parse::to_ir(src, self.font_size, style)
            .map_err(|e| Error::Parse(e.0))?;
        let b = boxer::layout(&node, style);
        Ok(svg::emit(&b, self.color))
    }
}

impl Default for MathRenderer {
    fn default() -> Self {
        MathRenderer::new()
    }
}

/// Render LaTeX `src` as inline (text-style) math, returning an Iced widget.
///
/// ```
/// let _el = iced_math::inline::<(), iced::Theme, iced::Renderer>("E = mc^2");
/// ```
pub fn inline<'a, Message, Theme, Renderer>(src: &str) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: svg_widget::Catalog + text::Catalog + 'a,
    <Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, Theme>>,
    Renderer: SvgRenderer + TextRenderer<Font = iced::Font> + 'a,
{
    build(MathRenderer::new(), src)
}

/// Render LaTeX `src` as a centered display-style block, returning an Iced widget.
///
/// ```
/// let _el = iced_math::block::<(), iced::Theme, iced::Renderer>(r"\frac{a}{b}");
/// ```
pub fn block<'a, Message, Theme, Renderer>(src: &str) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: svg_widget::Catalog + text::Catalog + container::Catalog + 'a,
    <Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, Theme>>,
    Renderer: SvgRenderer + TextRenderer<Font = iced::Font> + 'a,
{
    let el = build::<Message, Theme, Renderer>(MathRenderer::new().display_style(true), src);
    container(el).center_x(Length::Fill).padding(8).into()
}

fn build<'a, Message, Theme, Renderer>(
    renderer: MathRenderer,
    src: &str,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: svg_widget::Catalog + text::Catalog + 'a,
    <Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, Theme>>,
    Renderer: SvgRenderer + TextRenderer<Font = iced::Font> + 'a,
{
    match renderer.to_svg(src) {
        Ok(bytes) => widget::from_svg(bytes),
        Err(_) => widget::error_fallback(src.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_svg_default_is_black_no_group() {
        let bytes = MathRenderer::new().to_svg("x").unwrap();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.starts_with("<svg"));
        assert!(!s.contains("<g fill"), "default black must not wrap in a group");
    }

    #[test]
    fn to_svg_non_default_color_wraps_group() {
        let bytes = MathRenderer::new()
            .color(Color::rgb(0x11, 0x22, 0x33))
            .to_svg("x")
            .unwrap();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains(r##"<g fill="#112233">"##));
    }

    #[test]
    fn to_svg_rejects_bad_font_size() {
        for bad in [0.0_f32, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let r = MathRenderer::new().font_size(bad);
            assert!(matches!(r.to_svg("x"), Err(Error::InvalidFontSize(_))));
        }
    }
}
