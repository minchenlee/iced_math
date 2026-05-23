//! TeX inter-atom spacing table (TeXbook p. 170 / KaTeX `spacingData.js`).
//!
//! Given a pair of adjacent atom classes and the current style class
//! (display/text vs. script/scriptscript), returns the inter-atom spacing.
//! `Med` and `Thick` are suppressed in script and scriptscript styles per
//! TeX rules; `Thin` is always emitted.

use crate::ir::AtomClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spacing {
    None,
    Thin, // 3mu
    Med,  // 4mu
    Thick, // 5mu
}

impl Spacing {
    /// Convert to pixels given the current em (font size in px).
    pub fn to_px(self, font_size: f32) -> f32 {
        let em = font_size;
        match self {
            Spacing::None => 0.0,
            Spacing::Thin => em * 3.0 / 18.0,
            Spacing::Med => em * 4.0 / 18.0,
            Spacing::Thick => em * 5.0 / 18.0,
        }
    }
}

/// Stable class index — avoids relying on enum-discriminant ABI.
fn idx(c: AtomClass) -> usize {
    match c {
        AtomClass::Ord => 0,
        AtomClass::Op => 1,
        AtomClass::Bin => 2,
        AtomClass::Rel => 3,
        AtomClass::Open => 4,
        AtomClass::Close => 5,
        AtomClass::Punct => 6,
        AtomClass::Inner => 7,
    }
}

// 8×8 table indexed [left][right]. Source: KaTeX `src/spacingData.js`
// (canonical digital encoding of TeXbook p. 170).
//
// Cells where TeX states the combination "must not occur" (e.g. Bin–Bin)
// fall through to None — no crash, just no extra space.
const T: [[Spacing; 8]; 8] = {
    use Spacing::*;
    [
        //               Ord    Op     Bin    Rel    Open   Close  Punct  Inner
        /* Ord   */ [None,  Thin,  Med,   Thick, None,  None,  None,  Thin ],
        /* Op    */ [Thin,  Thin,  None,  Thick, None,  None,  None,  Thin ],
        /* Bin   */ [Med,   Med,   None,  None,  Med,   None,  None,  Med  ],
        /* Rel   */ [Thick, Thick, None,  None,  Thick, None,  None,  Thick],
        /* Open  */ [None,  None,  None,  None,  None,  None,  None,  None ],
        /* Close */ [None,  Thin,  Med,   Thick, None,  None,  None,  Thin ],
        /* Punct */ [Thin,  Thin,  None,  Thin,  Thin,  Thin,  Thin,  Thin ],
        /* Inner */ [Thin,  Thin,  Med,   Thick, Thin,  None,  Thin,  Thin ],
    ]
};

/// Compute inter-atom spacing.
///
/// `display_or_text` should be `true` for Display/Text styles and `false`
/// for Script/ScriptScript — in the latter case `Med` and `Thick` collapse
/// to `None` per TeX rules. `Thin` is always preserved.
pub fn between(left: AtomClass, right: AtomClass, display_or_text: bool) -> Spacing {
    let raw = T[idx(left)][idx(right)];
    if !display_or_text && matches!(raw, Spacing::Med | Spacing::Thick) {
        Spacing::None
    } else {
        raw
    }
}
