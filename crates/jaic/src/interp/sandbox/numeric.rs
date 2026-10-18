//! C-locale subject scanning and direct binary32/binary64 conversion, and the `strtod` and
//! `strtol` families built on it.

use super::*;

pub(super) struct Parsed {
    pub bits: u64,
    pub used: usize,
    pub range_error: bool,
}

/// Width of the floating-point value a conversion produces.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Precision {
    Binary32,
    Binary64,
}

pub(super) fn parse(text: &str, precision: Precision) -> Parsed {
    let narrow = precision == Precision::Binary32;
    let b = text.as_bytes();
    let mut at = 0;
    while b
        .get(at)
        .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 11 | 12))
    {
        at += 1;
    }
    let start = at;
    let negative = b.get(at) == Some(&b'-');
    if matches!(b.get(at), Some(b'+' | b'-')) {
        at += 1;
    }
    let (sign, infinity, nan) = if narrow {
        (1u64 << 31, 0x7f80_0000, 0x7fc0_0000)
    } else {
        (1u64 << 63, 0x7ff0_0000_0000_0000, 0x7ff8_0000_0000_0000)
    };
    let sign = if negative {
        sign
    } else {
        0
    };
    if text[at..]
        .get(..3)
        .is_some_and(|s| s.eq_ignore_ascii_case("inf"))
    {
        at += 3;
        if text[at..]
            .get(..5)
            .is_some_and(|s| s.eq_ignore_ascii_case("inity"))
        {
            at += 5;
        }
        return Parsed {
            bits: sign | infinity,
            used: at,
            range_error: false,
        };
    }
    if text[at..]
        .get(..3)
        .is_some_and(|s| s.eq_ignore_ascii_case("nan"))
    {
        at += 3;
        if b.get(at) == Some(&b'(') {
            let mut end = at + 1;
            while b
                .get(end)
                .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
            {
                end += 1;
            }
            if b.get(end) == Some(&b')') {
                at = end + 1;
            }
        }
        return Parsed {
            bits: sign | nan,
            used: at,
            range_error: false,
        };
    }
    let hex = b.get(at) == Some(&b'0')
        && matches!(b.get(at + 1), Some(b'x' | b'X'))
        && (b.get(at + 2).is_some_and(u8::is_ascii_hexdigit)
            || (b.get(at + 2) == Some(&b'.') && b.get(at + 3).is_some_and(u8::is_ascii_hexdigit)));
    if hex {
        at += 2;
    }
    let mantissa = at;
    let mut digits = 0;
    let mut point = false;
    let mut fractional = 0i64;
    let mut nonzero = false;
    while let Some(&byte) = b.get(at) {
        if byte == b'.' && !point {
            point = true;
            at += 1;
            continue;
        }
        if !(if hex {
            byte.is_ascii_hexdigit()
        } else {
            byte.is_ascii_digit()
        }) {
            break;
        }
        digits += 1;
        nonzero |= byte != b'0';
        if point {
            fractional = fractional.saturating_add(1);
        }
        at += 1;
    }
    if digits == 0 {
        return Parsed {
            bits: 0,
            used: 0,
            range_error: false,
        };
    }
    let mantissa_end = at;
    let mut exponent = 0i64;
    if b.get(at).is_some_and(|b| {
        if hex {
            matches!(b, b'p' | b'P')
        } else {
            matches!(b, b'e' | b'E')
        }
    }) {
        let mut end = at + 1;
        let minus = b.get(end) == Some(&b'-');
        if matches!(b.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let first = end;
        while let Some(&digit) = b.get(end).filter(|b| b.is_ascii_digit()) {
            exponent = exponent
                .saturating_mul(10)
                .saturating_add(i64::from(digit - b'0'));
            end += 1;
        }
        if end > first {
            at = end;
            if minus {
                exponent = -exponent;
            }
        } else {
            exponent = 0;
        }
    }
    let bits = if hex {
        sign | hex_bits(
            &b[mantissa..mantissa_end],
            exponent.saturating_sub(fractional.saturating_mul(4)),
            narrow,
        )
    } else if narrow {
        u64::from(
            text[start..at]
                .parse::<f32>()
                .expect("scanned decimal syntax")
                .to_bits(),
        )
    } else {
        text[start..at]
            .parse::<f64>()
            .expect("scanned decimal syntax")
            .to_bits()
    };
    let magnitude = bits & !sign;
    // C11 permits an implementation-defined underflow errno policy. This host
    // signals rounded-to-zero underflow; subnormal results preserve errno.
    Parsed {
        bits,
        used: at,
        range_error: magnitude == infinity || (magnitude == 0 && nonzero),
    }
}

fn hex_bits(digits: &[u8], scale: i64, narrow: bool) -> u64 {
    let mut prefix = 0u64;
    let mut stored = 0u32;
    let mut length = 0i64;
    let mut sticky = false;
    for &byte in digits {
        if byte == b'.' {
            continue;
        }
        let nibble = if byte.is_ascii_digit() {
            byte - b'0'
        } else {
            byte.to_ascii_lowercase() - b'a' + 10
        };
        if length == 0 && nibble == 0 {
            continue;
        }
        let width = if length == 0 {
            8 - nibble.leading_zeros()
        } else {
            4
        };
        length = length.saturating_add(i64::from(width));
        let take = width.min(64 - stored);
        prefix = (prefix << take) | u64::from(nibble >> (width - take));
        sticky |= nibble & ((1u8 << (width - take)) - 1) != 0;
        stored += take;
    }
    if length == 0 {
        return 0;
    }
    let (fraction, bias, infinity) = if narrow {
        (23i64, 127i64, 0x7f80_0000)
    } else {
        (52, 1023, 0x7ff0_0000_0000_0000)
    };
    let mut exponent = length.saturating_sub(1).saturating_add(scale);
    if exponent > bias {
        return infinity;
    }
    let minimum = 1 - bias;
    if exponent < minimum - fraction - 1 {
        return 0;
    }
    let unit = (exponent - fraction).max(minimum - fraction);
    let wanted = (exponent - unit + 1) as u32;
    let mut quotient = if wanted >= stored {
        prefix << (wanted - stored)
    } else if wanted == 0 {
        0
    } else {
        prefix >> (stored - wanted)
    };
    if wanted < stored {
        let guard_at = stored - wanted - 1;
        let guard = (prefix >> guard_at) & 1 != 0;
        let below = sticky || prefix & ((1u64 << guard_at) - 1) != 0;
        if guard && (below || quotient & 1 != 0) {
            quotient += 1;
        }
    }
    if exponent < minimum {
        return quotient;
    }
    if quotient == 1u64 << (fraction + 1) {
        quotient >>= 1;
        exponent += 1;
    }
    if exponent > bias {
        return infinity;
    }
    ((exponent + bias) as u64) << fraction | (quotient & ((1u64 << fraction) - 1))
}

impl SandboxHost {
    /// `atof`/`strtod` and the `atoi`/`strtol` family.
    pub(super) fn numeric_call(&mut self, fun: LibcFn, arg: impl Fn(usize) -> u64) -> Option<u64> {
        Some(match fun {
            LibcFn::Atof | LibcFn::Strtod | LibcFn::Strtof => {
                let text = cstr(arg(0));
                let parsed = parse(
                    &text,
                    if fun == LibcFn::Strtof {
                        Precision::Binary32
                    } else {
                        Precision::Binary64
                    },
                );
                if fun != LibcFn::Atof && arg(1) != 0 {
                    write_u64(arg(1), arg(0) + parsed.used as u64);
                }
                if parsed.range_error {
                    self.set_errno(34);
                }
                parsed.bits
            }
            LibcFn::Atoi
            | LibcFn::Atol
            | LibcFn::Atoll
            | LibcFn::Strtol
            | LibcFn::Strtoll
            | LibcFn::Strtoul
            | LibcFn::Strtoull => {
                let text = cstr(arg(0));
                let from_ascii = matches!(fun, LibcFn::Atoi | LibcFn::Atol | LibcFn::Atoll);
                let base = if from_ascii {
                    10
                } else {
                    arg(2) as u32
                };
                let (value, used) = parse_int_prefix(&text, base);
                if !from_ascii && arg(1) != 0 {
                    write_u64(arg(1), arg(0) + used as u64);
                }
                value as i64 as u64
            }
            _ => return None,
        })
    }
}

/// `strtol` family: returns (value, bytes consumed).
fn parse_int_prefix(text: &str, base: u32) -> (i128, usize) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let mut negative = false;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        negative = bytes[i] == b'-';
        i += 1;
    }
    let mut base = base;
    let has_hex_prefix =
        |at: usize| at + 1 < bytes.len() && bytes[at] == b'0' && (bytes[at + 1] | 0x20) == b'x';
    if (base == 0 || base == 16)
        && has_hex_prefix(i)
        && bytes.get(i + 2).is_some_and(u8::is_ascii_hexdigit)
    {
        i += 2;
        base = 16;
    } else if base == 0 {
        base = if bytes.get(i) == Some(&b'0') {
            8
        } else {
            10
        };
    }
    let first = i;
    let mut value: i128 = 0;
    while i < bytes.len() {
        let Some(digit) = (bytes[i] as char).to_digit(base) else {
            break;
        };
        value = value
            .saturating_mul(base as i128)
            .saturating_add(digit as i128);
        i += 1;
    }
    if i == first {
        return (0, 0);
    }
    let value = if negative {
        -value
    } else {
        value
    };
    (value, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_numeric_prefixes_signs_and_incomplete_exponents() {
        for (text, bits, used) in [
            ("\u{b}\t -1.5e+oops", (-1.5f64).to_bits(), 7),
            (" +.oops", 0, 0),
            ("0x.p2", 0, 1),
            ("-0x1p-", (-1.0f64).to_bits(), 4),
            ("1,5", 1.0f64.to_bits(), 1),
            ("\u{a0}1", 0, 0),
        ] {
            let value = parse(text, Precision::Binary64);
            assert_eq!((value.bits, value.used), (bits, used), "{text:?}");
        }
    }

    #[test]
    fn c_numeric_hexadecimal_rounding_and_subnormal_boundaries() {
        for (text, narrow, bits) in [
            ("0x1.000001p0", true, 0x3f80_0000),
            ("0x1.00000100000000000000000000000001p0", true, 0x3f80_0001),
            ("0x1p-149", true, 1),
            ("0x1p-150", true, 0),
            ("0x1.00001p-150", true, 1),
            ("0x1p-1074", false, 1),
            ("0x1p-1075", false, 0),
            ("0x1.fffffffffffffp1023", false, 0x7fef_ffff_ffff_ffff),
            ("0x1.fffffffffffff8p1023", false, 0x7ff0_0000_0000_0000),
        ] {
            assert_eq!(
                parse(
                    text,
                    if narrow {
                        Precision::Binary32
                    } else {
                        Precision::Binary64
                    }
                )
                .bits,
                bits,
                "{text}"
            );
        }
    }

    #[test]
    fn c_numeric_hexadecimal_preserves_exact_binary_values() {
        // Construct exact hexadecimal subjects directly from IEEE fields. This
        // covers arbitrary exponents/signs without another floating parser.
        let mut seed = 0x6d25_39e9_7c19_b5a1_u64;
        for narrow in [false, true] {
            let (fraction, bias, exponent_mask, sign_mask) = if narrow {
                (23, 127, 0xff, 1u64 << 31)
            } else {
                (52, 1023, 0x7ff, 1u64 << 63)
            };
            for _ in 0..2048 {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                let bits = if narrow {
                    seed & 0xffff_ffff
                } else {
                    seed
                };
                let exponent = (bits >> fraction) & exponent_mask;
                if exponent == exponent_mask {
                    continue;
                }
                let mut mantissa = bits & ((1u64 << fraction) - 1);
                let scale = if exponent == 0 {
                    1 - bias - fraction
                } else {
                    mantissa |= 1u64 << fraction;
                    exponent as i32 - bias - fraction
                };
                let sign = if bits & sign_mask != 0 {
                    "-"
                } else {
                    ""
                };
                let text = format!("{sign}0x{mantissa:x}p{scale}");
                let parsed = parse(
                    &text,
                    if narrow {
                        Precision::Binary32
                    } else {
                        Precision::Binary64
                    },
                );
                assert_eq!(parsed.bits, bits, "{text}");
                assert_eq!(parsed.used, text.len());
                assert!(!parsed.range_error, "{text}");
            }
        }
    }

    #[test]
    fn c_numeric_special_tokens_and_implemented_range_policy() {
        assert_eq!(parse("-nan()tail", Precision::Binary32).bits, 0xffc0_0000);
        assert_eq!(parse("nan(bad!)", Precision::Binary64).used, 3);
        assert_eq!(parse("+INFINITY!", Precision::Binary64).used, 9);
        for text in ["inf", "nan", "0e-99999", "0x0p-99999", "0x1p-1074"] {
            assert!(!parse(text, Precision::Binary64).range_error, "{text}");
        }
        for text in ["1e99999", "1e-99999", "-0x1p-99999"] {
            assert!(parse(text, Precision::Binary64).range_error, "{text}");
        }
    }
}
