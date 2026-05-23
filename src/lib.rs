//! Native LaTeX math widget for Iced 0.14.
//!
//! Two entry points:
//! - [`inline`] for in-line math (text-style sizing).
//! - [`block`] for display math (centered, larger ops with limits above/below).
//!
//! Returns an [`iced::Element`] you can drop into any view tree.

pub(crate) static FONT_BYTES: &[u8] = include_bytes!("../assets/LatinModernMath.otf");

pub mod boxer;
pub mod font;
pub mod ir;
pub mod parse;
pub mod spacing;
pub mod svg;
mod widget;

use iced::advanced::svg::Renderer as SvgRenderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::widget::{container, svg as svg_widget, text};
use iced::{Element, Length};

/// Render LaTeX `src` as inline (text-style) math.
pub fn inline<'a, Message, Theme, Renderer>(src: &str) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: svg_widget::Catalog + text::Catalog + 'a,
    <Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, Theme>>,
    Renderer: SvgRenderer + TextRenderer<Font = iced::Font> + 'a,
{
    build(src, ir::Style::Text)
}

/// Render LaTeX `src` as a centered display-style block.
pub fn block<'a, Message, Theme, Renderer>(src: &str) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: svg_widget::Catalog + text::Catalog + container::Catalog + 'a,
    <Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, Theme>>,
    Renderer: SvgRenderer + TextRenderer<Font = iced::Font> + 'a,
{
    let el = build::<Message, Theme, Renderer>(src, ir::Style::Display);
    container(el).center_x(Length::Fill).padding(8).into()
}

fn build<'a, Message, Theme, Renderer>(
    src: &str,
    style: ir::Style,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: svg_widget::Catalog + text::Catalog + 'a,
    <Theme as text::Catalog>::Class<'a>: From<text::StyleFn<'a, Theme>>,
    Renderer: SvgRenderer + TextRenderer<Font = iced::Font> + 'a,
{
    match parse::to_ir(src, 16.0, style) {
        Ok(ir) => {
            let b = boxer::layout(&ir, style);
            let bytes = svg::emit(&b);
            widget::from_svg(bytes)
        }
        Err(_) => widget::error_fallback(src.to_string()),
    }
}
