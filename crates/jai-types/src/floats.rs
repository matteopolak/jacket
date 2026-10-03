//! Typed IEEE values. Key equality preserves bits; numeric comparison is explicit.
use crate::{FloatType, Integer, IntegerType, Relation};
use std::{cmp::Ordering, fmt};

/// IEEE bit patterns, including signed zero and individual NaN payloads.
/// `Eq` and `Hash` are constant-key identity, not floating-point numeric equality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatValue {
    F32(u32),
    F64(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

/// Explicit fractional policy, independent of unverified Jai cast corner cases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatToIntMode {
    Exact,
    Truncate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatError {
    InvalidDecimal,
    LiteralOutOfRange(FloatType),
    WidthMismatch {
        left: FloatType,
        right: FloatType,
    },
    NonFiniteToInteger {
        value: FloatValue,
        target: IntegerType,
    },
    FractionalToInteger {
        value: FloatValue,
        target: IntegerType,
    },
    IntegerOutOfRange {
        value: FloatValue,
        target: IntegerType,
    },
    IntegerPrecisionLoss {
        value: Integer,
        target: FloatType,
    },
    NonFiniteConversion(FloatValue),
    FloatOverflow {
        value: FloatValue,
        target: FloatType,
    },
    FloatPrecisionLoss {
        value: FloatValue,
        target: FloatType,
    },
}
impl fmt::Display for FloatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDecimal => {
                f.write_str("invalid normalized decimal floating-point literal")
            }
            Self::LiteralOutOfRange(ty) => write!(f, "decimal literal overflows {ty:?}"),
            Self::WidthMismatch { left, right } => {
                write!(f, "floating-point widths differ: {left:?} and {right:?}")
            }
            Self::NonFiniteToInteger { target, .. } => {
                write!(f, "NaN or infinity cannot convert to {target:?}")
            }
            Self::FractionalToInteger { target, .. } => {
                write!(f, "fractional value cannot convert exactly to {target:?}")
            }
            Self::IntegerOutOfRange { target, .. } => write!(
                f,
                "floating-point value is outside {target:?}'s integer range"
            ),
            Self::IntegerPrecisionLoss { target, .. } => {
                write!(f, "integer cannot be represented exactly as {target:?}")
            }
            Self::NonFiniteConversion(_) => {
                f.write_str("exact floating-point conversion requires a finite value")
            }
            Self::FloatOverflow { target, .. } => {
                write!(f, "floating-point conversion overflows {target:?}")
            }
            Self::FloatPrecisionLoss { target, .. } => {
                write!(f, "floating-point conversion loses precision in {target:?}")
            }
        }
    }
}
impl std::error::Error for FloatError {}

impl FloatType {
    pub const fn bits(self) -> u32 {
        match self {
            Self::F32 => 32,
            Self::F64 => 64,
        }
    }
}

impl FloatValue {
    pub fn from_f32(value: f32) -> Self {
        Self::F32(value.to_bits())
    }
    pub fn from_f64(value: f64) -> Self {
        Self::F64(value.to_bits())
    }
    pub const fn ty(self) -> FloatType {
        match self {
            Self::F32(_) => FloatType::F32,
            Self::F64(_) => FloatType::F64,
        }
    }
    /// F32 bits occupy the low 32 bits. Use `ty()` together with this value.
    pub const fn bits(self) -> u64 {
        match self {
            Self::F32(bits) => bits as u64,
            Self::F64(bits) => bits,
        }
    }
    pub fn f32_value(self) -> Option<f32> {
        match self {
            Self::F32(bits) => Some(f32::from_bits(bits)),
            Self::F64(_) => None,
        }
    }
    pub fn f64_value(self) -> Option<f64> {
        match self {
            Self::F32(_) => None,
            Self::F64(bits) => Some(f64::from_bits(bits)),
        }
    }
    /// Numerically widens F32; raw NaN identity remains available through `bits()`.
    pub fn to_f64(self) -> f64 {
        match self {
            Self::F32(bits) => f64::from(f32::from_bits(bits)),
            Self::F64(bits) => f64::from_bits(bits),
        }
    }
    pub const fn is_nan(self) -> bool {
        match self {
            Self::F32(bits) => bits & 0x7fff_ffff > 0x7f80_0000,
            Self::F64(bits) => bits & 0x7fff_ffff_ffff_ffff > 0x7ff0_0000_0000_0000,
        }
    }
    pub const fn is_infinite(self) -> bool {
        match self {
            Self::F32(bits) => bits & 0x7fff_ffff == 0x7f80_0000,
            Self::F64(bits) => bits & 0x7fff_ffff_ffff_ffff == 0x7ff0_0000_0000_0000,
        }
    }
    pub const fn is_finite(self) -> bool {
        !self.is_nan() && !self.is_infinite()
    }
    pub const fn is_sign_negative(self) -> bool {
        match self {
            Self::F32(bits) => bits & 0x8000_0000 != 0,
            Self::F64(bits) => bits & 0x8000_0000_0000_0000 != 0,
        }
    }
    /// Unary negation changes only the sign bit, including for signaling NaNs.
    pub const fn negate(self) -> Self {
        match self {
            Self::F32(bits) => Self::F32(bits ^ 0x8000_0000),
            Self::F64(bits) => Self::F64(bits ^ 0x8000_0000_0000_0000),
        }
    }
    pub const fn abs(self) -> Self {
        match self {
            Self::F32(bits) => Self::F32(bits & 0x7fff_ffff),
            Self::F64(bits) => Self::F64(bits & 0x7fff_ffff_ffff_ffff),
        }
    }
    /// Parses the frontend's unsigned decimal spelling directly into its chosen width.
    /// Underscores/signs/NaN/infinity are not normalized decimal source spellings.
    pub fn parse_decimal(ty: FloatType, spelling: &str) -> Result<Self, FloatError> {
        if !spelling
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_digit() || *byte == b'.')
        {
            return Err(FloatError::InvalidDecimal);
        }
        let value = match ty {
            FloatType::F32 => Self::from_f32(
                spelling
                    .parse::<f32>()
                    .map_err(|_| FloatError::InvalidDecimal)?,
            ),
            FloatType::F64 => Self::from_f64(
                spelling
                    .parse::<f64>()
                    .map_err(|_| FloatError::InvalidDecimal)?,
            ),
        };
        if value.is_finite() {
            Ok(value)
        } else {
            Err(FloatError::LiteralOutOfRange(ty))
        }
    }
    pub fn binary(self, op: FloatOp, right: Self) -> Result<Self, FloatError> {
        match (self, right) {
            (Self::F32(left), Self::F32(right)) => {
                let (left, right) = (f32::from_bits(left), f32::from_bits(right));
                Ok(Self::from_f32(match op {
                    FloatOp::Add => left + right,
                    FloatOp::Subtract => left - right,
                    FloatOp::Multiply => left * right,
                    FloatOp::Divide => left / right,
                    FloatOp::Remainder => left % right,
                }))
            }
            (Self::F64(left), Self::F64(right)) => {
                let (left, right) = (f64::from_bits(left), f64::from_bits(right));
                Ok(Self::from_f64(match op {
                    FloatOp::Add => left + right,
                    FloatOp::Subtract => left - right,
                    FloatOp::Multiply => left * right,
                    FloatOp::Divide => left / right,
                    FloatOp::Remainder => left % right,
                }))
            }
            _ => Err(FloatError::WidthMismatch {
                left: self.ty(),
                right: right.ty(),
            }),
        }
    }
    pub fn partial_compare(self, right: Self) -> Result<Option<Ordering>, FloatError> {
        match (self, right) {
            (Self::F32(left), Self::F32(right)) => {
                Ok(f32::from_bits(left).partial_cmp(&f32::from_bits(right)))
            }
            (Self::F64(left), Self::F64(right)) => {
                Ok(f64::from_bits(left).partial_cmp(&f64::from_bits(right)))
            }
            _ => Err(FloatError::WidthMismatch {
                left: self.ty(),
                right: right.ty(),
            }),
        }
    }
    /// Numeric IEEE relations: signed zeros compare equal; NaNs are unordered.
    pub fn compare(self, relation: Relation, right: Self) -> Result<bool, FloatError> {
        let ordering = self.partial_compare(right)?;
        Ok(match relation {
            Relation::Equal => ordering == Some(Ordering::Equal),
            Relation::NotEqual => ordering != Some(Ordering::Equal),
            Relation::Less => ordering == Some(Ordering::Less),
            Relation::LessEqual => matches!(ordering, Some(Ordering::Less | Ordering::Equal)),
            Relation::Greater => ordering == Some(Ordering::Greater),
            Relation::GreaterEqual => matches!(ordering, Some(Ordering::Greater | Ordering::Equal)),
        })
    }
    /// Explicit IEEE width conversion; finite overflow can produce infinity.
    /// Same-width conversion preserves every bit; NaN payload conversion is host-defined.
    pub fn convert(self, target: FloatType) -> Self {
        match (self, target) {
            (Self::F32(bits), FloatType::F64) => Self::from_f64(f64::from(f32::from_bits(bits))),
            (Self::F64(bits), FloatType::F32) => Self::from_f32(f64::from_bits(bits) as f32),
            _ => self,
        }
    }
    pub fn cast(self, target: FloatType) -> Self {
        self.convert(target)
    }
    /// A strict finite, lossless policy; callers decide which source casts require it.
    pub fn convert_exact(self, target: FloatType) -> Result<Self, FloatError> {
        if !self.is_finite() {
            return Err(FloatError::NonFiniteConversion(self));
        }
        let converted = self.convert(target);
        if !converted.is_finite() {
            return Err(FloatError::FloatOverflow {
                value: self,
                target,
            });
        }
        if converted.to_f64() != self.to_f64() {
            return Err(FloatError::FloatPrecisionLoss {
                value: self,
                target,
            });
        }
        Ok(converted)
    }
    /// Direct integer-to-selected-width rounding, without an intermediate F64.
    pub fn from_integer(target: FloatType, value: Integer) -> Self {
        match target {
            FloatType::F32 => Self::from_f32(value.value() as f32),
            FloatType::F64 => Self::from_f64(value.value() as f64),
        }
    }
    pub fn from_integer_exact(target: FloatType, value: Integer) -> Result<Self, FloatError> {
        let converted = Self::from_integer(target, value);
        if converted.to_integer(value.ty(), FloatToIntMode::Exact) != Ok(value) {
            return Err(FloatError::IntegerPrecisionLoss { value, target });
        }
        Ok(converted)
    }
    /// Rejects nonfinite/out-of-range inputs; fractional loss requires explicit Truncate.
    pub fn to_integer(
        self,
        target: IntegerType,
        mode: FloatToIntMode,
    ) -> Result<Integer, FloatError> {
        if !self.is_finite() {
            return Err(FloatError::NonFiniteToInteger {
                value: self,
                target,
            });
        }
        let value = self.to_f64();
        if mode == FloatToIntMode::Exact && value.trunc() != value {
            return Err(FloatError::FractionalToInteger {
                value: self,
                target,
            });
        }
        // Every Jai integer fits within i128. Rust's finite float-to-i128 saturation
        // remains outside all supported ranges, so it cannot turn overflow into success.
        Integer::checked(target, value as i128).ok_or(FloatError::IntegerOutOfRange {
            value: self,
            target,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn bit_identity_preserves_width_signed_zero_and_nan_payloads() {
        let values = [
            FloatValue::F32(0),
            FloatValue::F32(0x8000_0000),
            FloatValue::F64(0),
            FloatValue::F32(0x7fbf_ffff),
            FloatValue::F32(0x7fc0_0001),
            FloatValue::F64(0x7ff8_0000_0000_0001),
        ];
        let keys = values.into_iter().collect::<HashSet<_>>();
        assert_eq!(keys.len(), values.len());
        for value in values {
            assert!(keys.contains(&value));
            assert_eq!(value.negate().negate(), value);
            assert_eq!(
                value.negate().bits(),
                value.bits() ^ (1 << (value.ty().bits() - 1))
            );
        }
        assert_eq!(
            FloatValue::F32(0xffbf_ffff).abs(),
            FloatValue::F32(0x7fbf_ffff)
        );
        assert_eq!(FloatValue::F32(1).bits(), 1);
        assert!(FloatValue::F32(1).is_finite());
        assert!(FloatValue::F64(0x7ff0_0000_0000_0000).is_infinite());
    }

    #[test]
    fn numerical_relations_are_separate_from_constant_key_equality() {
        for (zero, negative_zero, nan) in [
            (
                FloatValue::F32(0),
                FloatValue::F32(0x8000_0000),
                FloatValue::F32(0x7fbf_ffff),
            ),
            (
                FloatValue::F64(0),
                FloatValue::F64(0x8000_0000_0000_0000),
                FloatValue::F64(0x7ff8_0000_0000_0001),
            ),
        ] {
            assert_ne!(zero, negative_zero);
            assert!(zero.compare(Relation::Equal, negative_zero).unwrap());
            assert!(!nan.compare(Relation::Equal, nan).unwrap());
            assert!(nan.compare(Relation::NotEqual, nan).unwrap());
            for relation in [
                Relation::Less,
                Relation::LessEqual,
                Relation::Greater,
                Relation::GreaterEqual,
            ] {
                assert!(!nan.compare(relation, zero).unwrap());
            }
        }
    }

    #[test]
    fn decimal_context_rounds_directly_into_the_selected_width() {
        let spelling = "1.0000000596046448";
        assert_eq!(
            FloatValue::parse_decimal(FloatType::F32, spelling).unwrap(),
            FloatValue::F32(1.0_f32.to_bits() + 1)
        );
        let wide = FloatValue::parse_decimal(FloatType::F64, spelling).unwrap();
        assert_eq!(wide.convert(FloatType::F32), FloatValue::from_f32(1.0));
        assert_eq!(
            FloatValue::parse_decimal(FloatType::F32, ".5").unwrap(),
            FloatValue::from_f32(0.5)
        );
        assert_eq!(
            FloatValue::parse_decimal(FloatType::F32, "1e-1000").unwrap(),
            FloatValue::F32(0)
        );
        assert_eq!(
            FloatValue::parse_decimal(FloatType::F32, "1e1000"),
            Err(FloatError::LiteralOutOfRange(FloatType::F32))
        );
        for spelling in ["NaN", "inf", "-1.0", "+1.0", "1_0.0", "1e", " 1.0"] {
            assert_eq!(
                FloatValue::parse_decimal(FloatType::F64, spelling),
                Err(FloatError::InvalidDecimal)
            );
        }
    }

    #[test]
    fn operations_keep_their_width_and_ieee_special_results() {
        let one = FloatValue::from_f32(1.0);
        let zero = FloatValue::F32(0);
        assert!(one.binary(FloatOp::Divide, zero).unwrap().is_infinite());
        assert!(zero.binary(FloatOp::Divide, zero).unwrap().is_nan());
        assert_eq!(
            FloatValue::from_f32(16_777_216.0)
                .binary(FloatOp::Add, one)
                .unwrap(),
            FloatValue::from_f32(16_777_216.0)
        );
        assert_eq!(
            FloatValue::from_f64(16_777_216.0)
                .binary(FloatOp::Add, FloatValue::from_f64(1.0))
                .unwrap(),
            FloatValue::from_f64(16_777_217.0)
        );
        assert_eq!(
            FloatValue::from_f64(5.5)
                .binary(FloatOp::Remainder, FloatValue::from_f64(2.0))
                .unwrap(),
            FloatValue::from_f64(1.5)
        );
        assert!(matches!(
            one.binary(FloatOp::Add, FloatValue::F64(0)),
            Err(FloatError::WidthMismatch { .. })
        ));
        assert!(matches!(
            one.compare(Relation::Equal, FloatValue::F64(0)),
            Err(FloatError::WidthMismatch { .. })
        ));
    }

    #[test]
    fn float_integer_conversions_check_exact_power_of_two_boundaries() {
        for target in [
            IntegerType::S8,
            IntegerType::S16,
            IntegerType::S32,
            IntegerType::S64,
            IntegerType::U8,
            IntegerType::U16,
            IntegerType::U32,
            IntegerType::U64,
        ] {
            let upper = if target.signed() {
                1_u128 << (target.bits() - 1)
            } else {
                1_u128 << target.bits()
            };
            let outside = FloatValue::from_f64(upper as f64);
            assert!(matches!(
                outside.to_integer(target, FloatToIntMode::Exact),
                Err(FloatError::IntegerOutOfRange { .. })
            ));
            if target.signed() {
                assert_eq!(
                    outside
                        .negate()
                        .to_integer(target, FloatToIntMode::Exact)
                        .unwrap()
                        .value(),
                    target.min()
                );
            }
        }
        let below_u64 = FloatValue::F64((2_f64.powi(64)).to_bits() - 1);
        assert_eq!(
            below_u64
                .to_integer(IntegerType::U64, FloatToIntMode::Exact)
                .unwrap()
                .value(),
            (1_i128 << 64) - 2048
        );
        assert_eq!(
            FloatValue::from_f64(-128.9)
                .to_integer(IntegerType::S8, FloatToIntMode::Truncate)
                .unwrap()
                .value(),
            -128
        );
        assert!(matches!(
            FloatValue::from_f64(-128.9).to_integer(IntegerType::S8, FloatToIntMode::Exact),
            Err(FloatError::FractionalToInteger { .. })
        ));
        for value in [
            FloatValue::F32(0x7fbf_ffff),
            FloatValue::from_f64(f64::INFINITY),
            FloatValue::from_f64(f64::NEG_INFINITY),
        ] {
            assert!(matches!(
                value.to_integer(IntegerType::U64, FloatToIntMode::Truncate),
                Err(FloatError::NonFiniteToInteger { .. })
            ));
        }
        assert!(matches!(
            FloatValue::from_f64(f64::MAX).to_integer(IntegerType::U64, FloatToIntMode::Truncate),
            Err(FloatError::IntegerOutOfRange { .. })
        ));
    }

    #[test]
    fn exact_conversion_policy_reports_precision_and_overflow() {
        let integer = Integer::checked(IntegerType::U64, 16_777_217).unwrap();
        assert_eq!(
            FloatValue::from_integer(FloatType::F32, integer),
            FloatValue::from_f32(16_777_216.0)
        );
        assert!(matches!(
            FloatValue::from_integer_exact(FloatType::F32, integer),
            Err(FloatError::IntegerPrecisionLoss { .. })
        ));
        assert!(FloatValue::from_integer_exact(FloatType::F64, integer).is_ok());
        assert!(matches!(
            FloatValue::from_f64(0.1).convert_exact(FloatType::F32),
            Err(FloatError::FloatPrecisionLoss { .. })
        ));
        assert!(matches!(
            FloatValue::from_f64(f64::MAX).convert_exact(FloatType::F32),
            Err(FloatError::FloatOverflow { .. })
        ));
        assert!(matches!(
            FloatValue::from_f64(f64::NAN).convert_exact(FloatType::F64),
            Err(FloatError::NonFiniteConversion(_))
        ));
        assert_eq!(
            FloatValue::F64(0x8000_0000_0000_0000)
                .convert_exact(FloatType::F32)
                .unwrap(),
            FloatValue::F32(0x8000_0000)
        );
        let nan = FloatValue::F64(0x7ff0_0000_0000_0001);
        assert_eq!(nan.convert(FloatType::F64), nan);
    }
}
