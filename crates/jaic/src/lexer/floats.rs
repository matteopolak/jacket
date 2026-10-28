//! Float literal precision checks.

/// Default precision of a floating literal.
///
/// A decimal literal counts its fractional digits (trailing zeros ignored, integer digits
/// excluded): eight or more default to `float64`, fewer to `float32`. More than eight
/// significant ones (leading zeros of a literal below one do not count, so `0.000000001` is
/// one digit) require `float64`, as does a normal binary exponent outside -126..=127.
/// This is independent of whether rounding to `float32` happens to preserve the value.
///
/// A `0h` bit-pattern literal is `float64` when it has 16 hex digits and `float32` otherwise;
/// the checker rejects widths other than 4, 8 and 16.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FloatLiteralPrecision {
    pub defaults_to_float64: bool,
    pub requires_float64: bool,
    pub requires_float64_due_to_significant_digits: bool,
}

pub fn has_big_float32_exponent(value: f64) -> bool {
    let encoded = ((value.to_bits() >> 52) & 0x7ff) as i32;
    encoded != 0 && encoded != 0x7ff && !(-126..=127).contains(&(encoded - 1023))
}

pub fn float_literal_precision(text: &str, value: f64) -> FloatLiteralPrecision {
    if let Some(hex) = text.strip_prefix("0h").or_else(|| text.strip_prefix("0H")) {
        return FloatLiteralPrecision {
            defaults_to_float64: hex.bytes().filter(u8::is_ascii_hexdigit).count() == 16,
            ..Default::default()
        };
    }
    let mantissa = text.split(['e', 'E']).next().unwrap_or("").replace('_', "");
    let fractional = mantissa
        .split_once('.')
        .map_or("", |(_, fraction)| fraction);
    let fractional = fractional.trim_end_matches('0');
    let counted = fractional.len();
    let integer_part_zero = mantissa
        .split('.')
        .next()
        .unwrap_or("")
        .bytes()
        .all(|b| b == b'0');
    let significant = if integer_part_zero {
        fractional.trim_start_matches('0').len()
    } else {
        counted
    } > 8;
    let required = significant || has_big_float32_exponent(value);
    FloatLiteralPrecision {
        defaults_to_float64: !required && counted >= 8,
        requires_float64: required,
        requires_float64_due_to_significant_digits: significant,
    }
}
