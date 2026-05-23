//! Bundled math font parsing.

use std::sync::OnceLock;
use ttf_parser::Face;

use crate::FONT_BYTES;

fn face() -> &'static Face<'static> {
    static FACE: OnceLock<Face<'static>> = OnceLock::new();
    FACE.get_or_init(|| {
        Face::parse(FONT_BYTES, 0).expect("bundled math font must parse")
    })
}

pub fn units_per_em() -> f32 {
    face().units_per_em() as f32
}

pub fn has_math_table() -> bool {
    face().tables().math.is_some()
}
