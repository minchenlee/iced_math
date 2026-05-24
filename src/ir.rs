//! Intermediate representation between pulldown-latex parse events and the boxer.

use ttf_parser::GlyphId;

// `Space` and `Error` variants are reserved for later tiers (explicit spacing
// nodes, structured parse errors) and not constructed yet.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Atom {
        class: AtomClass,
        glyph: GlyphId,
        font_size: f32,
    },
    Frac {
        num: Box<Node>,
        den: Box<Node>,
        /// Draw the horizontal fraction rule. `false` for `\binom`/`\atop`
        /// (ruleless stack), `true` for ordinary `\frac`.
        bar: bool,
    },
    Subsup {
        base: Box<Node>,
        sub: Option<Box<Node>>,
        sup: Option<Box<Node>>,
    },
    Radical {
        degree: Option<Box<Node>>,
        body: Box<Node>,
    },
    Row(Vec<Node>),
    Fenced {
        open: GlyphId,
        close: GlyphId,
        body: Box<Node>,
    },
    Op {
        glyph: GlyphId,
        limits: bool,
        big: bool,
        font_size: f32,
    },
    /// Multi-letter operator name (`\sin`, `\log`, `\lim`). Rendered upright and
    /// tightly set (no inter-letter math spacing), but behaves as a single `Op`
    /// atom for inter-atom spacing. `limits: true` (the `\lim`/`\max` family)
    /// stacks scripts above/below in display style; otherwise scripts attach to
    /// the side like an ordinary base.
    OpName {
        body: Box<Node>,
        limits: bool,
    },
    /// A math accent (`\hat`, `\vec`, `\bar`, `\tilde`, …): `accent` is a
    /// combining-accent glyph centered horizontally over `body` and raised to
    /// sit just above it.
    Accent {
        accent: GlyphId,
        body: Box<Node>,
    },
    /// A matrix / aligned / cases grid. `rows[r][c]` is the cell node at row `r`,
    /// column `c` (ragged rows allowed — short rows are treated as empty on the
    /// right). `col_aligns` gives each column's horizontal alignment; columns
    /// beyond its length default to `ColAlign::Center`. Any enclosing delimiters
    /// (e.g. `pmatrix`'s parens) are applied by a surrounding [`Node::Fenced`].
    Matrix {
        rows: Vec<Vec<Node>>,
        col_aligns: Vec<ColAlign>,
    },
    Space(SpaceKind),
    /// Sentinel for parse error — boxer emits red-monospace fallback.
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomClass {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Inner,
}

// Reserved for explicit-spacing support (`\,` `\;` `\quad` `\mkern`) in a later tier.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpaceKind {
    Thin,
    Med,
    Thick,
    NegThin,
    /// Width supplied in design-unit-style multiplier (mu = 1/18 em). 6mu = thin, 4mu = med-neg, etc.
    Mu(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Display,
    Text,
    Script,
    ScriptScript,
}

impl Style {
    pub fn is_display(self) -> bool {
        matches!(self, Style::Display)
    }
    pub fn sub(self) -> Self {
        match self {
            Style::Display | Style::Text => Style::Script,
            Style::Script | Style::ScriptScript => Style::ScriptScript,
        }
    }
    pub fn font_size(self, base: f32) -> f32 {
        match self {
            Style::Display | Style::Text => base,
            Style::Script => base * 0.7,
            Style::ScriptScript => base * 0.5,
        }
    }
}
