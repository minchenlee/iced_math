//! Public error type returned by the low-level rendering API.

/// Errors that can occur while rendering LaTeX math to SVG.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The LaTeX source failed to parse, or contained a construct with no
    /// glyph / unsupported at this version. Carries a human-readable message.
    Parse(String),
    /// The configured font size was not a finite, strictly-positive number
    /// (e.g. `0.0`, negative, `NaN`, or infinite). Carries the offending value.
    InvalidFontSize(f32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(msg) => write!(f, "math parse error: {msg}"),
            Error::InvalidFontSize(v) => {
                write!(f, "invalid font size: {v} (must be finite and > 0)")
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_message() {
        let e = Error::Parse("boom".into());
        assert_eq!(e.to_string(), "math parse error: boom");
    }

    #[test]
    fn display_invalid_font_size() {
        let e = Error::InvalidFontSize(-1.0);
        assert!(e.to_string().contains("invalid font size"));
    }
}
