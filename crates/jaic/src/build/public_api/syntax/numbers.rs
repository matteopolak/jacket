//! Bit and precision preserving floating literal serialization.
use super::Printer;
use crate::{interp::Trap, types::TypeId};

impl Printer<'_, '_> {
    /// Preserve both value bits and the lexer's precision category, as `Program_Print` does.
    pub(super) fn floating(&self, at: u64, ty: TypeId, bits: u64) -> Result<String, Trap> {
        let value = f64::from_bits(bits);
        if self.has(at, ty, "value_flags", "HEX")? {
            let negative = self.has(at, ty, "value_flags", "MINUS_SIGN")?;
            let magnitude = if negative {
                -value
            } else {
                value
            };
            let text = if self.has(at, ty, "value_flags", "DEFAULTS_TO_FLOAT64")? {
                format!("0h{:016x}", magnitude.to_bits())
            } else {
                let narrow = magnitude as f32;
                if f64::from(narrow).to_bits() != magnitude.to_bits() {
                    return Err(Trap::new(
                        "edited float literal is not representable at its declared precision",
                    ));
                }
                format!("0h{:08x}", narrow.to_bits())
            };
            return Ok(format!(
                "{}{text}",
                if negative {
                    "-"
                } else {
                    ""
                }
            ));
        }
        if value.is_nan() {
            return Err(Trap::new("cannot serialize a decimal NaN literal"));
        }
        let wanted = if self.has(at, ty, "value_flags", "REQUIRES_FLOAT64")? {
            9
        } else if self.has(at, ty, "value_flags", "DEFAULTS_TO_FLOAT64")? {
            8
        } else {
            0
        };
        let sign = if value.is_sign_negative() {
            "-"
        } else {
            ""
        };
        if value.is_infinite() {
            return Ok(format!(
                "{sign}{}e999",
                if wanted == 9 {
                    "1.000000001"
                } else {
                    "1.0"
                }
            ));
        }
        if value == 0.0 && wanted > 0 {
            return Ok(format!("{sign}0.{}1e-999", "0".repeat(wanted - 1)));
        }
        let scientific = format!("{:.16e}", value.abs());
        let (coefficient, exponent) = scientific.split_once('e').unwrap();
        let fraction = coefficient
            .split_once('.')
            .map_or(0, |(_, part)| part.len());
        let mut exponent = exponent.parse::<i32>().unwrap() - fraction as i32;
        let mut digits = coefficient.replace('.', "");
        while digits.len() > 1 && digits.ends_with('0') {
            digits.pop();
            exponent += 1;
        }
        let text = if wanted == 0 {
            format!("{digits}.0")
        } else if digits.len() <= wanted {
            format!("0.{}{digits}", "0".repeat(wanted - digits.len()))
        } else {
            let split = digits.len() - wanted;
            format!("{}.{}", &digits[..split], &digits[split..])
        };
        Ok(format!("{sign}{text}e{}", exponent + wanted as i32))
    }
}
