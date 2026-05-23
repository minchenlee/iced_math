//! Iced widget wrappers for rendered math.
//!
//! Two helpers:
//! - [`from_svg`] — wrap rasterizable SVG bytes into an iced [`svg::Svg`] widget.
//! - [`error_fallback`] — render the original (failed) LaTeX source as red
//!   monospace text, so the user sees what broke instead of an empty hole.
//!
//! Both return an [`Element`] generic over `Theme`/`Renderer`/`Message` so the
//! caller can drop the result anywhere in their view tree.

use iced::advanced::svg::Renderer as SvgRenderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::widget::svg as svg_widget;
use iced::widget::text;
use iced::{Color, Element, Font, Length};

/// Wrap freshly emitted SVG bytes into an iced [`svg::Svg`] widget.
///
/// The widget uses `Length::Shrink` on both axes so it lays out exactly at the
/// SVG's declared viewport size — important for inline math sitting on a text
/// baseline.
pub(crate) fn from_svg<'a, Message, Theme, Renderer>(
    bytes: Vec<u8>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: svg_widget::Catalog + 'a,
    Renderer: SvgRenderer + 'a,
{
    svg_widget::Svg::new(svg_widget::Handle::from_memory(bytes))
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
}

/// Render the original LaTeX source as a red monospace fallback when parsing
/// or layout fails.
///
/// We deliberately keep this visible (vs. silently empty) so authors see the
/// broken snippet inline and can fix it.
pub(crate) fn error_fallback<'a, Message, Theme, Renderer>(
    src: String,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: text::Catalog + 'a,
    Renderer: TextRenderer<Font = Font> + 'a,
    <Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, Theme>>,
{
    text::Text::new(src)
        .font(Font::MONOSPACE)
        .color(Color::from_rgb8(0xc0, 0x39, 0x2b))
        .into()
}
