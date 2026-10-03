//! Type identities and checked values shared by compiler phases.
mod build;
pub use build::*;
mod floats;
pub use floats::*;
mod operators;
pub use operators::*;
mod cast_modifiers;
pub use cast_modifiers::{CastModifier, CastModifiers, CastModifiersError};
mod storage_bitcast;
pub use storage_bitcast::{StorageBitcast, StorageBitcastError, StorageBitcastStrength};
mod safety_checks;
pub use safety_checks::CheckMode;
mod inline_hints;
pub use inline_hints::InlineHint;
mod execution_phase;
pub use execution_phase::ProcedureExecution;
mod debug_policy;
pub use debug_policy::DebugPolicy;
mod registry;
pub use registry::*;
mod any;
pub use any::*;
mod allocator;
pub use allocator::*;
mod runtime_types;
pub use runtime_types::*;
mod runtime_info;
pub use runtime_info::*;
mod layout;
pub use layout::*;
mod reflection;
pub use reflection::*;
mod record_reflection;
pub use record_reflection::*;
mod code_values;
pub use code_values::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntegerType {
    S8,
    S16,
    S32,
    S64,
    U8,
    U16,
    U32,
    U64,
}
impl IntegerType {
    pub fn bits(self) -> u32 {
        match self {
            Self::S8 | Self::U8 => 8,
            Self::S16 | Self::U16 => 16,
            Self::S32 | Self::U32 => 32,
            Self::S64 | Self::U64 => 64,
        }
    }
    pub fn signed(self) -> bool {
        matches!(self, Self::S8 | Self::S16 | Self::S32 | Self::S64)
    }
    pub fn min(self) -> i128 {
        if self.signed() {
            -(1i128 << (self.bits() - 1))
        } else {
            0
        }
    }
    pub fn max(self) -> i128 {
        (1i128 << (self.bits() - u32::from(self.signed()))) - 1
    }
    pub fn contains(self, other: Self) -> bool {
        self.min() <= other.min() && self.max() >= other.max()
    }
    pub fn common(self, other: Self) -> Option<Self> {
        if self.contains(other) {
            Some(self)
        } else if other.contains(self) {
            Some(other)
        } else {
            None
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScalarType {
    Int(IntegerType),
    Bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnType {
    Void,
    Value(ScalarType),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CastMode {
    Checked,
    Unchecked,
    Truncate,
    Force(StorageBitcastStrength),
}
/// A normalized fixed-width bit pattern. Construction states whether loss is allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Integer {
    ty: IntegerType,
    bits: u64,
}
impl Integer {
    pub fn checked(ty: IntegerType, value: i128) -> Option<Self> {
        (ty.min()..=ty.max())
            .contains(&value)
            .then(|| Self::wrapping(ty, value))
    }
    pub fn wrapping(ty: IntegerType, value: i128) -> Self {
        let mask = u64::MAX >> (64 - ty.bits());
        Self {
            ty,
            bits: value as u64 & mask,
        }
    }
    pub fn ty(self) -> IntegerType {
        self.ty
    }
    pub fn bits(self) -> u64 {
        self.bits
    }
    pub fn value(self) -> i128 {
        if self.ty.signed() {
            let shift = 64 - self.ty.bits();
            ((self.bits << shift) as i64 >> shift) as i128
        } else {
            self.bits as i128
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_width_has_correct_limits_and_normalization() {
        for ty in [
            IntegerType::S8,
            IntegerType::S16,
            IntegerType::S32,
            IntegerType::S64,
            IntegerType::U8,
            IntegerType::U16,
            IntegerType::U32,
            IntegerType::U64,
        ] {
            for value in [ty.min(), 0, ty.max()] {
                assert_eq!(Integer::checked(ty, value).unwrap().value(), value);
            }
            assert!(Integer::checked(ty, ty.min() - 1).is_none());
            assert!(Integer::checked(ty, ty.max() + 1).is_none());
            assert_eq!(Integer::wrapping(ty, ty.max() + 1).value(), ty.min());
        }
    }
    #[test]
    fn implicit_conversion_requires_containment_of_the_entire_range() {
        assert!(IntegerType::S64.contains(IntegerType::U32));
        assert!(IntegerType::U16.contains(IntegerType::U8));
        assert!(!IntegerType::U64.contains(IntegerType::S8));
        assert!(!IntegerType::S64.contains(IntegerType::U64));
        assert_eq!(IntegerType::S8.common(IntegerType::U8), None);
    }
}
