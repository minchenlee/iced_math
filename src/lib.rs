//! Native LaTeX math widget for Iced 0.14.

pub(crate) static FONT_BYTES: &[u8] = include_bytes!("../assets/LatinModernMath.otf");

pub mod boxer;
pub mod font;
pub mod ir;
pub mod parse;
pub mod spacing;
