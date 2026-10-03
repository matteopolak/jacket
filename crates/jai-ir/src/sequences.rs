use crate::{IrError, ValueExpr};
use jai_types::{IntegerType, ScalarType, TypeId, TypeKind, TypeView};

/// Cumulative scratch backing budget for ordered variadic concatenation per frame.
pub const MAX_SEQUENCE_TEMP_BYTES: u64 = 1_048_576;

/// Conservative bookkeeping and stack padding charged for each backing region.
pub const SEQUENCE_TEMP_ALLOCATION_OVERHEAD: u64 = 64;

/// Sequence indices retain signedness while extending without range loss.
pub fn canonical_index_type(source: IntegerType) -> IntegerType {
    if source.signed() {
        IntegerType::S64
    } else {
        IntegerType::U64
    }
}

/// Shared VM/native resource charge for nonempty caller-frame pack storage.
/// Callers bypass empty regions; nonempty zero-sized elements own a sentinel.
pub fn sequence_temp_allocation_charge(bytes: u64, alignment: u32) -> Option<u64> {
    if !alignment.is_power_of_two() {
        return None;
    }
    bytes
        .max(1)
        .checked_add(u64::from(alignment.max(16) - 1))?
        .checked_add(SEQUENCE_TEMP_ALLOCATION_OVERHEAD)
}

#[derive(Clone, Debug)]
pub enum SequencePackPart {
    Element(ValueExpr),
    Spread(ValueExpr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SequenceField {
    Count,
    Data,
    Allocated,
}
impl SequenceField {
    pub fn index(self) -> usize {
        match self {
            Self::Count => 0,
            Self::Data => 1,
            Self::Allocated => 2,
        }
    }
}
pub(crate) fn element(types: &dyn TypeView, ty: TypeId) -> Result<TypeId, IrError> {
    match *types.kind(ty)? {
        TypeKind::FixedArray { element, .. }
        | TypeKind::Slice(element)
        | TypeKind::DynamicArray(element)
        | TypeKind::Pointer(element) => Ok(element),
        TypeKind::String => Ok(types.scalar(ScalarType::Int(IntegerType::U8))),
        _ => Err(IrError::InvalidValue(ty)),
    }
}
pub(crate) fn field_type(
    types: &dyn TypeView,
    base: TypeId,
    field: SequenceField,
) -> Result<TypeId, IrError> {
    match types.kind(base)? {
        TypeKind::FixedArray { .. }
        | TypeKind::Slice(_)
        | TypeKind::DynamicArray(_)
        | TypeKind::String => {}
        _ => return Err(IrError::InvalidValue(base)),
    }
    match field {
        SequenceField::Count => Ok(types.scalar(ScalarType::Int(IntegerType::S64))),
        SequenceField::Allocated if matches!(types.kind(base)?, TypeKind::DynamicArray(_)) => {
            Ok(types.scalar(ScalarType::Int(IntegerType::S64)))
        }
        SequenceField::Allocated => Err(IrError::InvalidValue(base)),
        SequenceField::Data => types
            .lookup(&TypeKind::Pointer(element(types, base)?))
            .ok_or(IrError::InvalidValue(base)),
    }
}

#[cfg(test)]
mod allocation_charge_tests {
    use super::*;
    #[test]
    fn nonempty_storage_includes_zero_size_padding_and_metadata() {
        assert_eq!(sequence_temp_allocation_charge(0, 1), Some(80));
        assert_eq!(sequence_temp_allocation_charge(8, 8), Some(87));
        assert_eq!(sequence_temp_allocation_charge(8, 32), Some(103));
    }
    #[test]
    fn invalid_alignment_and_arithmetic_overflow_are_rejected() {
        assert_eq!(sequence_temp_allocation_charge(1, 0), None);
        assert_eq!(sequence_temp_allocation_charge(1, 3), None);
        assert_eq!(sequence_temp_allocation_charge(u64::MAX, 16), None);
    }
}
