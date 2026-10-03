//! Target-independent integer casts whose native result is an address constant.
//! These values do not grant VM data or code provenance.
use jai_types::{CastMode, Integer, IntegerType, TypeError, TypeId, TypeKind, TypeView};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativePointerSource {
    /// An untyped literal is normalized only against the selected target.
    Weak(i128),
    Strong(Integer),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NativePointerConstant {
    ty: TypeId,
    source: NativePointerSource,
    mode: CastMode,
}

#[derive(Debug)]
pub enum NativePointerConstantError {
    Type(TypeError),
    InvalidType(TypeId),
    UnsupportedWidth(u32),
    UnsupportedMode(CastMode),
    CheckedCast,
}
impl fmt::Display for NativePointerConstantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => error.fmt(formatter),
            Self::InvalidType(ty) => write!(
                formatter,
                "native address constant requires pointer type {ty:?}"
            ),
            Self::UnsupportedWidth(bits) => write!(
                formatter,
                "native address constant does not support {bits}-bit pointers"
            ),
            Self::UnsupportedMode(mode) => {
                write!(
                    formatter,
                    "native address constant does not support {mode:?} cast mode"
                )
            }
            Self::CheckedCast => formatter.write_str(
                "native address constant checked cast is out of range for the selected target",
            ),
        }
    }
}
impl std::error::Error for NativePointerConstantError {}
impl From<TypeError> for NativePointerConstantError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}

impl NativePointerConstant {
    fn numeric_mode(mode: CastMode) -> Result<(), NativePointerConstantError> {
        match mode {
            CastMode::Checked | CastMode::Unchecked | CastMode::Truncate => Ok(()),
            CastMode::Force(_) => Err(NativePointerConstantError::UnsupportedMode(mode)),
        }
    }

    pub fn new(
        ty: TypeId,
        source: Integer,
        mode: CastMode,
        types: &dyn TypeView,
    ) -> Result<Self, NativePointerConstantError> {
        Self::numeric_mode(mode)?;
        if !matches!(types.kind(ty)?, TypeKind::Pointer(_)) {
            return Err(NativePointerConstantError::InvalidType(ty));
        }
        Ok(Self {
            ty,
            source: NativePointerSource::Strong(source),
            mode,
        })
    }

    pub fn new_weak(
        ty: TypeId,
        source: i128,
        mode: CastMode,
        types: &dyn TypeView,
    ) -> Result<Self, NativePointerConstantError> {
        Self::numeric_mode(mode)?;
        if !matches!(types.kind(ty)?, TypeKind::Pointer(_)) {
            return Err(NativePointerConstantError::InvalidType(ty));
        }
        Ok(Self {
            ty,
            source: NativePointerSource::Weak(source),
            mode,
        })
    }

    pub fn type_id(&self) -> TypeId {
        self.ty
    }
    pub fn source(&self) -> &NativePointerSource {
        &self.source
    }
    pub fn mode(&self) -> CastMode {
        self.mode
    }

    pub fn validate(&self, types: &dyn TypeView) -> Result<(), NativePointerConstantError> {
        Self::numeric_mode(self.mode)?;
        if !matches!(types.kind(self.ty)?, TypeKind::Pointer(_)) {
            return Err(NativePointerConstantError::InvalidType(self.ty));
        }
        Ok(())
    }

    /// Normalize against the execution target, never Rust's host pointer width.
    /// Same-width signed casts preserve bits; smaller signed sources sign extend.
    pub fn address(&self, bits: u32) -> Result<Integer, NativePointerConstantError> {
        Self::numeric_mode(self.mode)?;
        let target = match bits {
            8 => IntegerType::U8,
            16 => IntegerType::U16,
            32 => IntegerType::U32,
            64 => IntegerType::U64,
            _ => return Err(NativePointerConstantError::UnsupportedWidth(bits)),
        };
        let maximum = u64::MAX >> (64 - bits);
        let address = match self.source {
            NativePointerSource::Weak(source) => {
                if self.mode == CastMode::Checked
                    && (source < -(1i128 << (bits - 1)) || source > i128::from(maximum))
                {
                    return Err(NativePointerConstantError::CheckedCast);
                }
                source as u64 & maximum
            }
            NativePointerSource::Strong(source) => {
                if source.ty().bits() > bits
                    && self.mode == CastMode::Checked
                    && (source.value() < 0 || source.value() > i128::from(maximum))
                {
                    return Err(NativePointerConstantError::CheckedCast);
                }
                if source.ty().bits() < bits {
                    source.value() as u64 & maximum
                } else {
                    source.bits() & maximum
                }
            }
        };
        Ok(Integer::wrapping(target, i128::from(address)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::TypeRegistry;

    #[test]
    fn address_casts_obey_the_selected_width_and_distinct_modes() {
        let mut types = TypeRegistry::new();
        let ty = types.pointer(types.void()).unwrap();
        let source = Integer::wrapping(IntegerType::S64, -1);
        let checked = NativePointerConstant::new(ty, source, CastMode::Checked, &types).unwrap();
        assert_eq!(checked.address(64).unwrap().bits(), u64::MAX);
        assert!(matches!(
            checked.address(32),
            Err(NativePointerConstantError::CheckedCast)
        ));
        for mode in [CastMode::Unchecked, CastMode::Truncate] {
            let pointer = NativePointerConstant::new(ty, source, mode, &types).unwrap();
            for bits in [8, 16, 32, 64] {
                assert_eq!(
                    pointer.address(bits).unwrap().bits(),
                    u64::MAX >> (64 - bits)
                );
            }
        }
    }

    #[test]
    fn smaller_signed_inputs_extend_and_wrong_kinds_or_widths_reject() {
        let mut types = TypeRegistry::new();
        let ty = types.pointer(types.void()).unwrap();
        let source = Integer::wrapping(IntegerType::S8, -1);
        let pointer = NativePointerConstant::new(ty, source, CastMode::Checked, &types).unwrap();
        assert_eq!(pointer.address(64).unwrap().bits(), u64::MAX);
        assert!(matches!(
            pointer.address(128),
            Err(NativePointerConstantError::UnsupportedWidth(128))
        ));
        assert!(matches!(
            NativePointerConstant::new(types.void(), source, CastMode::Checked, &types),
            Err(NativePointerConstantError::InvalidType(_))
        ));
    }

    #[test]
    fn checked_unsigned_boundaries_and_same_width_signed_values_are_exact() {
        let mut types = TypeRegistry::new();
        let ty = types.pointer(types.void()).unwrap();
        for value in [u64::MAX, 1_u64 << 32] {
            let source = Integer::wrapping(IntegerType::U64, i128::from(value));
            let pointer =
                NativePointerConstant::new(ty, source, CastMode::Checked, &types).unwrap();
            assert!(matches!(
                pointer.address(32),
                Err(NativePointerConstantError::CheckedCast)
            ));
        }
        let signed = NativePointerConstant::new(
            ty,
            Integer::wrapping(IntegerType::S32, -1),
            CastMode::Checked,
            &types,
        )
        .unwrap();
        assert_eq!(signed.address(32).unwrap().bits(), u64::from(u32::MAX));
        let unsigned = NativePointerConstant::new(
            ty,
            Integer::wrapping(IntegerType::U32, i128::from(u32::MAX)),
            CastMode::Checked,
            &types,
        )
        .unwrap();
        assert_eq!(unsigned.address(64).unwrap().bits(), u64::from(u32::MAX));
    }

    #[test]
    fn storage_force_cannot_enter_a_numeric_address_capsule() {
        let mut types = TypeRegistry::new();
        let ty = types.pointer(types.void()).unwrap();
        for strength in [
            jai_types::StorageBitcastStrength::EqualSize,
            jai_types::StorageBitcastStrength::Prefix,
        ] {
            let mode = CastMode::Force(strength);
            assert!(matches!(
                NativePointerConstant::new(
                    ty, Integer::wrapping(IntegerType::U64, 42), mode, &types,
                ),
                Err(NativePointerConstantError::UnsupportedMode(actual)) if actual == mode
            ));
        }
    }

    #[test]
    fn a_pointer_type_from_another_arena_cannot_be_certified() {
        let mut origin = TypeRegistry::new();
        let foreign_pointer = origin.pointer(origin.void()).unwrap();
        let unrelated = TypeRegistry::new();
        assert!(matches!(
            NativePointerConstant::new(
                foreign_pointer,
                Integer::wrapping(IntegerType::U64, 42),
                CastMode::Checked,
                &unrelated,
            ),
            Err(NativePointerConstantError::Type(_))
        ));
    }
}
