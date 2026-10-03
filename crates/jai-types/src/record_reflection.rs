//! Record metadata reductions never change physical types or storage.
use crate::{FieldId, TypeError, TypeId, TypeKind, TypeRegistry, TypeView};
use std::fmt;

mod transactions;
pub use transactions::{
    PreparedRecordReflectionTransaction, RecordReflectionChange, RecordReflectionCommit,
    RecordReflectionTransaction, RecordReflectionTransactionError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RecordReflectionFlag {
    NoTypeInfo = 1,
    ProceduresAreVoidPointers = 2,
    NoSizeComplaint = 4,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RecordReflectionPolicy(u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownRecordReflectionFlags {
    pub bits: u32,
}
impl fmt::Display for UnknownRecordReflectionFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown record reflection flag bits: {:#x}", self.bits)
    }
}
impl std::error::Error for UnknownRecordReflectionFlags {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordMemberReflection {
    Omitted,
    Declared(TypeId),
    VoidPointer,
}

impl RecordReflectionPolicy {
    pub fn from_flags(flags: impl IntoIterator<Item = RecordReflectionFlag>) -> Self {
        Self(flags.into_iter().fold(0, |bits, flag| bits | flag as u8))
    }
    pub fn from_bits(bits: u32) -> Result<Self, UnknownRecordReflectionFlags> {
        const KNOWN: u32 = RecordReflectionFlag::NoTypeInfo as u32
            | RecordReflectionFlag::ProceduresAreVoidPointers as u32
            | RecordReflectionFlag::NoSizeComplaint as u32;
        let unknown = bits & !KNOWN;
        if unknown != 0 {
            return Err(UnknownRecordReflectionFlags { bits: unknown });
        }
        Ok(Self(bits as u8))
    }
    pub fn bits(self) -> u32 {
        u32::from(self.0)
    }
    pub fn contains(self, flag: RecordReflectionFlag) -> bool {
        self.0 & flag as u8 != 0
    }
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub fn member_type(
        self,
        types: &dyn TypeView,
        record: TypeId,
        field: FieldId,
    ) -> Result<RecordMemberReflection, TypeError> {
        let ty = types.validate_field(record, field)?;
        if !matches!(types.kind(record)?, TypeKind::Record(_)) {
            return Err(TypeError::WrongKind(record));
        }
        if self.contains(RecordReflectionFlag::NoTypeInfo) {
            return Ok(RecordMemberReflection::Omitted);
        }
        if self.contains(RecordReflectionFlag::ProceduresAreVoidPointers)
            && matches!(types.kind(ty)?, TypeKind::Procedure(_))
        {
            return Ok(RecordMemberReflection::VoidPointer);
        }
        Ok(RecordMemberReflection::Declared(ty))
    }
}

impl TypeRegistry {
    pub fn add_record_reflection_flags(
        &mut self,
        record: TypeId,
        flags: RecordReflectionPolicy,
    ) -> Result<RecordReflectionPolicy, TypeError> {
        let current = self.record_reflection_policy(record)?;
        let policy = current.union(flags);
        if policy.contains(RecordReflectionFlag::ProceduresAreVoidPointers) {
            self.pointer(self.void())?;
        }
        self.set_record_reflection_policy(record, policy);
        Ok(policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CallingConvention, ContextMode, IntegerType, LayoutEngine, LayoutPolicy, ProcedureType,
        RecordKind, ScalarType, Variadic,
    };

    #[test]
    fn raw_flags_are_checked_and_combinations_are_monotone() {
        for bits in 0..8 {
            assert_eq!(
                RecordReflectionPolicy::from_bits(bits).unwrap().bits(),
                bits
            );
        }
        for bits in [8, 0x100, u32::MAX] {
            let error = RecordReflectionPolicy::from_bits(bits).unwrap_err();
            assert_eq!(error.bits, bits & !7);
        }
        let hidden = RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]);
        let pointers = RecordReflectionPolicy::from_flags([
            RecordReflectionFlag::ProceduresAreVoidPointers,
            RecordReflectionFlag::NoSizeComplaint,
        ]);
        assert_eq!(hidden.union(pointers).bits(), 7);
        assert_eq!(hidden.union(pointers).union(hidden).bits(), 7);
    }

    fn fixture() -> (TypeRegistry, TypeId, TypeId) {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let callback = types
            .procedure(ProcedureType {
                parameters: vec![int].into(),
                results: vec![int].into(),
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: Variadic::None,
            })
            .unwrap();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, vec![int, callback]).unwrap();
        (types, record, callback)
    }

    #[test]
    fn flags_preserve_nominal_fields_layout_and_frozen_policy() {
        let (mut types, record, callback) = fixture();
        let before = LayoutEngine::new(&types, LayoutPolicy::lp64())
            .layout(record)
            .unwrap()
            .clone();
        let scalar = types.field(record, 0).unwrap();
        let procedure = types.field(record, 1).unwrap();
        let reduced = types
            .add_record_reflection_flags(
                record,
                RecordReflectionPolicy::from_flags([
                    RecordReflectionFlag::ProceduresAreVoidPointers,
                ]),
            )
            .unwrap();
        assert_eq!(
            reduced.member_type(&types, record, scalar.id).unwrap(),
            RecordMemberReflection::Declared(scalar.ty)
        );
        assert_eq!(
            reduced.member_type(&types, record, procedure.id).unwrap(),
            RecordMemberReflection::VoidPointer
        );
        let suppressed = types
            .add_record_reflection_flags(
                record,
                RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoSizeComplaint]),
            )
            .unwrap();
        assert!(suppressed.contains(RecordReflectionFlag::ProceduresAreVoidPointers));
        assert_eq!(types.field_type(procedure.id).unwrap(), callback);
        assert_eq!(
            LayoutEngine::new(&types, LayoutPolicy::lp64())
                .layout(record)
                .unwrap()
                .clone(),
            before
        );
        let frozen = types.freeze().unwrap();
        assert_eq!(frozen.record_reflection_policy(record).unwrap(), suppressed);
        assert_eq!(frozen.field_type(procedure.id).unwrap(), callback);
    }

    #[test]
    fn hidden_members_do_not_weaken_owner_or_bounds_checks() {
        let (mut types, record, _) = fixture();
        let other = types.reserve_record(RecordKind::Struct);
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        types.define_record(other, vec![int]).unwrap();
        let hidden = types
            .add_record_reflection_flags(
                record,
                RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
            )
            .unwrap();
        assert_eq!(
            hidden
                .member_type(&types, record, types.field(record, 0).unwrap().id)
                .unwrap(),
            RecordMemberReflection::Omitted
        );
        assert!(matches!(
            hidden.member_type(&types, record, types.field(other, 0).unwrap().id),
            Err(TypeError::FieldOwner { .. })
        ));
        assert!(types.field(record, 2).is_err());
        assert_eq!(
            types.record_reflection_policy(other).unwrap(),
            RecordReflectionPolicy::default()
        );
    }

    #[test]
    fn source_reservations_accept_flags_without_adopting_foreign_or_structural_types() {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        let policy = RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]);
        types.add_record_reflection_flags(record, policy).unwrap();
        let mut foreign = TypeRegistry::new();
        let foreign_record = foreign.reserve_record(RecordKind::Struct);
        assert!(matches!(
            types.add_record_reflection_flags(foreign_record, policy),
            Err(TypeError::ForeignType(_))
        ));
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        assert!(matches!(
            types.add_record_reflection_flags(int, policy),
            Err(TypeError::WrongKind(_))
        ));
        assert_eq!(types.record_reflection_policy(record).unwrap(), policy);
        types.define_record(record, vec![int]).unwrap();
        assert_eq!(
            types
                .freeze()
                .unwrap()
                .record_reflection_policy(record)
                .unwrap(),
            policy
        );
    }

    #[test]
    fn descriptor_pointer_is_interned_only_after_nominal_validation() {
        let mut types = TypeRegistry::new();
        let pointer = TypeKind::Pointer(types.void());
        let policy =
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::ProceduresAreVoidPointers]);
        assert!(types.lookup(&pointer).is_none());
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        assert!(types.add_record_reflection_flags(int, policy).is_err());
        assert!(types.lookup(&pointer).is_none());
        let record = types.reserve_record(RecordKind::Struct);
        types.add_record_reflection_flags(record, policy).unwrap();
        let descriptor_pointer = types.lookup(&pointer).unwrap();
        types.add_record_reflection_flags(record, policy).unwrap();
        assert_eq!(types.lookup(&pointer), Some(descriptor_pointer));
        types.define_record(record, vec![int]).unwrap();
        let frozen = types.freeze().unwrap();
        assert_eq!(frozen.lookup(&pointer), Some(descriptor_pointer));
    }
}
