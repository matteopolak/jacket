//! Checked nominal storage and procedure contract for Preload's allocator.
use crate::{
    CallingConvention, ContextMode, FieldDescriptor, Integer, IntegerType, Layout, LayoutEngine,
    LayoutError, LayoutPolicy, RecordKind, ScalarType, TypeError, TypeId, TypeKind, TypeRegistry,
    TypeView, Variadic,
};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AllocatorField {
    Procedure,
    Data,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AllocatorMode {
    Allocate,
    Resize,
    Free,
    Startup,
    Shutdown,
    ThreadStart,
    ThreadStop,
    CreateHeap,
    DestroyHeap,
    IsThisYours,
    Capabilities,
}
impl AllocatorMode {
    pub const ALL: [Self; 11] = [
        Self::Allocate,
        Self::Resize,
        Self::Free,
        Self::Startup,
        Self::Shutdown,
        Self::ThreadStart,
        Self::ThreadStop,
        Self::CreateHeap,
        Self::DestroyHeap,
        Self::IsThisYours,
        Self::Capabilities,
    ];
    pub fn value(self) -> Integer {
        Integer::wrapping(IntegerType::S64, self as i128)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocatorSchema {
    ty: TypeId,
    mode: TypeId,
    procedure: FieldDescriptor,
    data: FieldDescriptor,
}

#[derive(Debug)]
pub enum AllocatorError {
    Type(TypeError),
    Layout(LayoutError),
    InvalidRecord(TypeId),
    InvalidMode(TypeId),
    InvalidProcedure(TypeId),
    InvalidData(TypeId),
    ConflictingBinding { expected: TypeId, actual: TypeId },
}
impl From<TypeError> for AllocatorError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl From<LayoutError> for AllocatorError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}
impl fmt::Display for AllocatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => error.fmt(f),
            Self::Layout(error) => error.fmt(f),
            Self::InvalidRecord(_) => {
                f.write_str("allocator must have canonical procedure/data struct storage")
            }
            Self::InvalidMode(_) => {
                f.write_str("allocator mode must have the canonical signed 64-bit operation values")
            }
            Self::InvalidProcedure(_) => {
                f.write_str("allocator procedure has an incompatible Jai signature")
            }
            Self::InvalidData(_) => f.write_str("allocator data must have void-pointer type"),
            Self::ConflictingBinding {
                ..
            } => f.write_str("allocator role is already bound to another nominal type"),
        }
    }
}
impl std::error::Error for AllocatorError {
}

impl AllocatorSchema {
    pub fn validate(
        types: &dyn TypeView,
        ty: TypeId,
        mode: TypeId,
    ) -> Result<Self, AllocatorError> {
        let record = types.record_definition(ty)?;
        if record.kind != RecordKind::Struct
            || record.fields.len() != 2
            || record.layout.packed
            || record.layout.minimum_alignment.is_some()
            || record.layout.field_alignments.iter().any(Option::is_some)
            || record.layout.field_placements.iter().any(Option::is_some)
        {
            return Err(AllocatorError::InvalidRecord(ty));
        }
        let modes = types.enum_definition(mode)?;
        if modes.representation != IntegerType::S64
            || modes.values.len() != AllocatorMode::ALL.len()
            || !modes
                .values
                .iter()
                .copied()
                .eq(AllocatorMode::ALL.map(AllocatorMode::value))
        {
            return Err(AllocatorError::InvalidMode(mode));
        }
        let procedure = types.field(ty, 0)?;
        let data = types.field(ty, 1)?;
        let TypeKind::Pointer(pointee) = *types.kind(data.ty)? else {
            return Err(AllocatorError::InvalidData(data.ty));
        };
        if !matches!(types.kind(pointee)?, TypeKind::Void) {
            return Err(AllocatorError::InvalidData(data.ty));
        }
        let signature = types.procedure_definition(procedure.ty)?;
        let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
        if signature.convention != CallingConvention::Jai
            || signature.context != ContextMode::Implicit
            || signature.variadic != Variadic::None
            || signature.parameters.as_ref() != [mode, s64, s64, data.ty, data.ty]
            || signature.results.as_ref() != [data.ty]
        {
            return Err(AllocatorError::InvalidProcedure(procedure.ty));
        }
        Ok(Self {
            ty,
            mode,
            procedure,
            data,
        })
    }
    pub fn ty(self) -> TypeId {
        self.ty
    }
    pub fn mode_type(self) -> TypeId {
        self.mode
    }
    pub fn field(self, field: AllocatorField) -> FieldDescriptor {
        match field {
            AllocatorField::Procedure => self.procedure,
            AllocatorField::Data => self.data,
        }
    }
    pub fn layout(
        self,
        types: &dyn TypeView,
        policy: LayoutPolicy,
    ) -> Result<Layout, AllocatorError> {
        if Self::validate(types, self.ty, self.mode)? != self {
            return Err(AllocatorError::InvalidRecord(self.ty));
        }
        let layout = LayoutEngine::new(types, policy).layout(self.ty)?.clone();
        let pointer = policy.pointer();
        let size = pointer
            .size
            .checked_mul(2)
            .ok_or(LayoutError::Overflow(self.ty))?;
        if layout.size != size
            || layout.alignment != pointer.alignment
            || layout.field_offsets.as_ref() != [0, pointer.size]
        {
            return Err(AllocatorError::InvalidRecord(self.ty));
        }
        Ok(layout)
    }
}

impl TypeRegistry {
    /// The source binder designates the authentic Preload declarations.
    pub fn bind_allocator(
        &mut self,
        ty: TypeId,
        mode: TypeId,
    ) -> Result<AllocatorSchema, AllocatorError> {
        let schema = AllocatorSchema::validate(self, ty, mode)?;
        if let Some(expected) = self.allocator_schema() {
            if expected != schema {
                return Err(AllocatorError::ConflictingBinding {
                    expected: expected.ty(),
                    actual: ty,
                });
            }
            return Ok(expected);
        }
        self.set_allocator_schema(schema);
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProcedureType, RecordLayout};

    fn mode(types: &mut TypeRegistry) -> TypeId {
        let mode = types.reserve_enum(IntegerType::S64);
        types
            .define_enum(mode, AllocatorMode::ALL.map(AllocatorMode::value))
            .unwrap();
        mode
    }
    fn signature(types: &mut TypeRegistry, mode: TypeId) -> ProcedureType {
        let data = types.pointer(types.void()).unwrap();
        let size = types.scalar(ScalarType::Int(IntegerType::S64));
        ProcedureType {
            parameters: Box::new([mode, size, size, data, data]),
            results: Box::new([data]),
            return_abi: crate::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        }
    }
    fn allocator(types: &mut TypeRegistry, signature: ProcedureType) -> TypeId {
        let data = signature.results[0];
        let procedure = types.procedure(signature).unwrap();
        let allocator = types.reserve_record(RecordKind::Struct);
        types.define_record(allocator, [procedure, data]).unwrap();
        allocator
    }

    #[test]
    fn binding_is_explicit_nominal_and_survives_freeze() {
        let mut types = TypeRegistry::new();
        let mode = mode(&mut types);
        let signature = signature(&mut types, mode);
        let allocator = allocator(&mut types, signature.clone());
        let lookalike = self::allocator(&mut types, signature);
        let proof = AllocatorSchema::validate(&types, allocator, mode).unwrap();
        assert_eq!(types.allocator_schema(), None);
        assert_eq!(types.bind_allocator(allocator, mode).unwrap(), proof);
        assert_eq!(types.bind_allocator(allocator, mode).unwrap(), proof);
        assert!(
            matches!(types.bind_allocator(lookalike, mode), Err(AllocatorError::ConflictingBinding { expected, actual }) if expected == allocator && actual == lookalike)
        );
        assert_eq!(types.allocator_schema(), Some(proof));
        assert_eq!(
            proof.field(AllocatorField::Procedure).id,
            types.field(allocator, 0).unwrap().id
        );
        assert_eq!(
            proof.field(AllocatorField::Data).id,
            types.field(allocator, 1).unwrap().id
        );
        let frozen = types.freeze().unwrap();
        assert_eq!(frozen.allocator_schema(), Some(proof));
        assert_eq!(
            AllocatorSchema::validate(&frozen, allocator, mode).unwrap(),
            proof
        );
    }

    #[test]
    fn layout_uses_selected_pointer_width() {
        let mut types = TypeRegistry::new();
        let mode = mode(&mut types);
        let signature = signature(&mut types, mode);
        let allocator = allocator(&mut types, signature);
        let proof = types.bind_allocator(allocator, mode).unwrap();
        let pointer32 = LayoutPolicy::new(
            crate::ScalarLayout::new(4, 4),
            [
                crate::ScalarLayout::new(1, 1),
                crate::ScalarLayout::new(2, 2),
                crate::ScalarLayout::new(4, 4),
                crate::ScalarLayout::new(8, 4),
            ],
            [
                crate::ScalarLayout::new(4, 4),
                crate::ScalarLayout::new(8, 4),
            ],
            crate::ScalarLayout::new(1, 1),
        )
        .unwrap();
        for policy in [LayoutPolicy::lp64(), pointer32] {
            let layout = proof.layout(&types, policy).unwrap();
            assert_eq!(layout.size, policy.pointer().size * 2);
            assert_eq!(layout.field_offsets.as_ref(), [0, policy.pointer().size]);
        }
    }

    #[test]
    fn foreign_or_incomplete_types_never_publish_a_role() {
        let mut types = TypeRegistry::new();
        let pending = types.reserve_record(RecordKind::Struct);
        let mode = mode(&mut types);
        assert!(
            matches!(types.bind_allocator(pending, mode), Err(AllocatorError::Type(TypeError::Incomplete(id))) if id == pending)
        );
        assert_eq!(types.allocator_schema(), None);
        let mut other = TypeRegistry::new();
        let other_mode = self::mode(&mut other);
        let signature = signature(&mut other, other_mode);
        let foreign = allocator(&mut other, signature);
        assert!(
            matches!(types.bind_allocator(foreign, other_mode), Err(AllocatorError::Type(TypeError::ForeignType(id))) if id == foreign)
        );
        assert_eq!(types.allocator_schema(), None);
    }

    #[test]
    fn procedure_contract_rejects_wrong_abi_context_and_slot_types() {
        for change in 0..5 {
            let mut types = TypeRegistry::new();
            let mode = mode(&mut types);
            let mut signature = signature(&mut types, mode);
            match change {
                0 => signature.convention = CallingConvention::C,
                1 => signature.context = ContextMode::None,
                2 => signature.parameters[1] = types.scalar(ScalarType::Int(IntegerType::U64)),
                3 => signature.parameters[0] = types.scalar(ScalarType::Int(IntegerType::S64)),
                4 => signature.results = Box::new([]),
                _ => unreachable!(),
            }
            let data = types.pointer(types.void()).unwrap();
            let procedure = types.procedure(signature).unwrap();
            let allocator = types.reserve_record(RecordKind::Struct);
            types.define_record(allocator, [procedure, data]).unwrap();
            assert!(matches!(
                types.bind_allocator(allocator, mode),
                Err(AllocatorError::InvalidProcedure(_))
            ));
            assert_eq!(types.allocator_schema(), None);
        }
    }

    #[test]
    fn invalid_mode_values_and_record_storage_are_rejected() {
        let mut types = TypeRegistry::new();
        let mode = types.reserve_enum(IntegerType::S64);
        let mut values = AllocatorMode::ALL.map(AllocatorMode::value);
        values[2] = Integer::wrapping(IntegerType::S64, 99);
        types.define_enum(mode, values).unwrap();
        let signature = signature(&mut types, mode);
        let allocator = allocator(&mut types, signature);
        assert!(
            matches!(types.bind_allocator(allocator, mode), Err(AllocatorError::InvalidMode(id)) if id == mode)
        );
        assert_eq!(types.allocator_schema(), None);
        let proper_mode = self::mode(&mut types);
        let signature = self::signature(&mut types, proper_mode);
        let data = signature.results[0];
        let procedure = types.procedure(signature).unwrap();
        let packed = types.reserve_record(RecordKind::Struct);
        types
            .define_record_with_layout(
                packed,
                [procedure, data],
                RecordLayout {
                    packed: true,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(
            matches!(types.bind_allocator(packed, proper_mode), Err(AllocatorError::InvalidRecord(id)) if id == packed)
        );
        assert_eq!(types.allocator_schema(), None);
    }

    #[test]
    fn explicit_empty_layout_metadata_has_the_same_storage_contract() {
        let mut types = TypeRegistry::new();
        let mode = mode(&mut types);
        let signature = signature(&mut types, mode);
        let data = signature.results[0];
        let procedure = types.procedure(signature).unwrap();
        let allocator = types.reserve_record(RecordKind::Struct);
        types
            .define_record_with_layout(
                allocator,
                [procedure, data],
                RecordLayout {
                    field_alignments: Box::new([None, None]),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(types.bind_allocator(allocator, mode).is_ok());
    }
}
