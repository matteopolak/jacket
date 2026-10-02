//! Decode integer digits directly from source without allocating a cleaned string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Error {
    MissingDigits,
    InvalidDigit,
    Overflow,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MissingDigits => "integer literal requires digits",
            Self::InvalidDigit => "expected an integer literal",
            Self::Overflow => "integer literal exceeds the u64 range",
        })
    }
}
pub(super) fn integer(text: &str) -> Result<u64, Error> {
    let (radix, digits) = if let Some(s) = text.strip_prefix("0x") {
        (16, s)
    } else if let Some(s) = text.strip_prefix("0b") {
        (2, s)
    } else {
        (10, text)
    };
    let mut value = 0u64;
    let mut seen = false;
    for byte in digits.bytes() {
        if byte == b'_' {
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return Err(Error::InvalidDigit),
        };
        if u64::from(digit) >= radix {
            return Err(Error::InvalidDigit);
        }
        value = value
            .checked_mul(radix)
            .and_then(|n| n.checked_add(u64::from(digit)))
            .ok_or(Error::Overflow)?;
        seen = true;
    }
    if seen {
        Ok(value)
    } else {
        Err(Error::MissingDigits)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_radices_and_separators() {
        for (text, value) in [
            ("0", 0),
            ("18_446_744_073_709_551_615", u64::MAX),
            ("0xffff_ffff_ffff_ffff", u64::MAX),
            ("0b101010", 42),
            ("0xFADE_DEAF_CAFE_BABE", 0xfade_deaf_cafe_babe),
        ] {
            assert_eq!(integer(text), Ok(value));
        }
        assert_eq!(integer("18446744073709551616"), Err(Error::Overflow));
        assert_eq!(integer("0x10000000000000000"), Err(Error::Overflow));
        assert_eq!(integer("0b2"), Err(Error::InvalidDigit));
        assert_eq!(integer("1.0"), Err(Error::InvalidDigit));
        assert_eq!(integer("0x_"), Err(Error::MissingDigits));
    }
}
