//! How a numeric literal was written.
use super::{ExprKind, Int128};

/// How a numeric literal was written, read once when it is parsed so no later check looks at
/// the source text again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Spelling {
    /// Plain digits, or digits with an exponent but no point: `255`, `1e9`.
    Decimal,
    /// A decimal number with a point: `1.5`, `.5`.
    Fractional,
    /// A hex or binary integer, or a `0h` float bit pattern: `0xff`, `0b101`, `0h3f800000`.
    Radix,
}

impl Spelling {
    /// The spelling of the literal `text`.
    pub fn of(text: &str) -> Self {
        let lower = text.as_bytes().get(..2).map(<[u8]>::to_ascii_lowercase);
        if matches!(lower.as_deref(), Some(b"0x" | b"0b" | b"0h")) {
            Self::Radix
        } else if text.contains('.') {
            Self::Fractional
        } else {
            Self::Decimal
        }
    }
}

impl ExprKind {
    /// An integer literal written as `text`.
    pub fn int_literal(value: u128, text: &str) -> Self {
        Self::Int(Int128::new(value), Spelling::of(text))
    }

    /// A float literal written as `text`.
    pub fn float_literal(value: f64, text: &str) -> Self {
        Self::Float(
            value,
            crate::lexer::float_literal_precision(text, value),
            Spelling::of(text),
        )
    }
}
